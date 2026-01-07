#!/usr/bin/env python3
"""
Gate Analyzer - Identifies donation-focused sites from Shopify gates
Analyzes URLs to find charity/donation sites suitable for card validation
"""

import requests
import re
from bs4 import BeautifulSoup
from rich.console import Console
from rich.table import Table
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich import box
import time
import json
from typing import List, Dict, Tuple

console = Console()

# Keywords that indicate donation/charity sites
DONATION_KEYWORDS = [
    'donate', 'donation', 'charity', 'foundation', 'nonprofit', 'non-profit',
    'fundrais', 'giving', 'support', 'contribute', 'help', 'cause',
    'relief', 'aid', 'mission', 'humanitarian', 'welfare', 'benevolent'
]

# Keywords that indicate e-commerce (to filter out)
ECOMMERCE_KEYWORDS = [
    'shop', 'store', 'buy', 'cart', 'product', 'clothing', 'fashion',
    'apparel', 'jewelry', 'accessories', 'shoes', 'bags', 'electronics'
]


def load_shopify_gates(directory: str) -> List[str]:
    """Load all Shopify gate URLs from text files"""
    import os
    import glob
    
    gates = []
    pattern = os.path.join(directory, '15000ShopifyGatescom_*.txt')
    
    for file_path in sorted(glob.glob(pattern)):
        try:
            with open(file_path, 'r') as f:
                urls = [line.strip() for line in f if line.strip()]
                gates.extend(urls)
        except Exception as e:
            console.print(f"[red]Error reading {file_path}: {e}[/]")
    
    return gates


def analyze_url_keywords(url: str) -> Tuple[bool, int]:
    """
    Analyze URL for donation keywords
    Returns (is_donation_site, keyword_score)
    """
    url_lower = url.lower()
    
    # Check for donation keywords in URL
    donation_score = sum(1 for keyword in DONATION_KEYWORDS if keyword in url_lower)
    
    # Check for e-commerce keywords (negative score)
    ecommerce_score = sum(1 for keyword in ECOMMERCE_KEYWORDS if keyword in url_lower)
    
    # If URL contains donation keywords and no e-commerce keywords
    is_donation = donation_score > 0 and ecommerce_score == 0
    
    return is_donation, donation_score


def check_site_content(url: str, timeout: int = 10) -> Dict:
    """
    Check site content for donation indicators with Shopify integration
    Returns dict with analysis results
    """
    result = {
        'url': url,
        'accessible': False,
        'has_shopify': False,
        'has_shopify_payments': False,
        'has_donation_form': False,
        'donation_keywords_count': 0,
        'shopify_domain': None,
        'payment_gateway': None,
        'title': None,
        'error': None
    }
    
    try:
        headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        }
        
        response = requests.get(url, headers=headers, timeout=timeout, allow_redirects=True)
        
        if response.status_code == 200:
            result['accessible'] = True
            
            soup = BeautifulSoup(response.text, 'html.parser')
            
            # Get page title
            title_tag = soup.find('title')
            if title_tag:
                result['title'] = title_tag.get_text().strip()
            
            page_text = response.text.lower()
            page_source = response.text
            
            # Check for Shopify (primary indicator)
            shopify_indicators = [
                'shopify' in page_text,
                'myshopify.com' in url.lower(),
                'cdn.shopify.com' in page_source,
                soup.find('meta', {'name': 'shopify-checkout-api-token'}),
                soup.find('script', {'src': re.compile(r'cdn\.shopify\.com', re.I)}),
                'shopify-features' in page_source,
                'shopify.theme' in page_source
            ]
            
            result['has_shopify'] = any(shopify_indicators)
            
            # Extract Shopify domain
            if 'myshopify.com' in url.lower():
                result['shopify_domain'] = url
            
            # Check for Shopify Payments integration
            shopify_payment_indicators = [
                'shopify payments' in page_text,
                'shopify.payment' in page_source,
                'checkout.shopify.com' in page_source,
                soup.find('form', {'action': re.compile(r'checkout\.shopify\.com', re.I)}),
                soup.find('input', {'name': 'checkout'}),
                'shopifypay' in page_text,
                'shop_pay' in page_text
            ]
            
            result['has_shopify_payments'] = any(shopify_payment_indicators)
            
            # Detect payment gateway (Shopify can use Stripe, but we want Shopify Payments)
            if 'shopify payments' in page_text or 'shop pay' in page_text:
                result['payment_gateway'] = 'Shopify Payments'
            elif 'stripe' in page_text and result['has_shopify']:
                result['payment_gateway'] = 'Shopify + Stripe'
            elif result['has_shopify']:
                result['payment_gateway'] = 'Shopify (Unknown Gateway)'
            
            # Check for donation form indicators
            donation_indicators = [
                soup.find('form', {'id': re.compile(r'donat', re.I)}),
                soup.find('form', {'class': re.compile(r'donat', re.I)}),
                soup.find('input', {'name': re.compile(r'donat|amount|contribution', re.I)}),
                soup.find('button', text=re.compile(r'donate|give|contribute', re.I)),
                soup.find('a', {'href': re.compile(r'donate|donation|contribute', re.I)}),
                soup.find('div', {'class': re.compile(r'donat', re.I)}),
                'donation' in page_text and 'amount' in page_text
            ]
            
            result['has_donation_form'] = any(indicator is not None for indicator in donation_indicators)
            
            # Count donation keywords in content
            result['donation_keywords_count'] = sum(
                1 for keyword in DONATION_KEYWORDS if keyword in page_text
            )
        else:
            result['error'] = f"HTTP {response.status_code}"
            
    except requests.exceptions.Timeout:
        result['error'] = "Timeout"
    except requests.exceptions.ConnectionError:
        result['error'] = "Connection Error"
    except Exception as e:
        result['error'] = str(e)[:50]
    
    return result


def analyze_gates(gates: List[str], max_check: int = 100) -> List[Dict]:
    """
    Analyze gates to find donation sites
    """
    console.print("\n[bold cyan]🔍 Analyzing Shopify Gates for Donation Sites[/bold cyan]\n")
    
    # Step 1: Quick URL analysis
    console.print("[yellow]Step 1: Analyzing URLs for donation keywords...[/yellow]")
    
    potential_donation_sites = []
    
    for url in gates:
        is_donation, score = analyze_url_keywords(url)
        if is_donation:
            potential_donation_sites.append({
                'url': url,
                'keyword_score': score
            })
    
    console.print(f"[green]✓ Found {len(potential_donation_sites)} potential donation sites from URL analysis[/green]\n")
    
    # Step 2: Check top candidates
    console.print(f"[yellow]Step 2: Checking top {min(max_check, len(potential_donation_sites))} sites for Shopify integration...[/yellow]\n")
    
    # Sort by keyword score
    potential_donation_sites.sort(key=lambda x: x['keyword_score'], reverse=True)
    
    verified_sites = []
    
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        console=console
    ) as progress:
        
        task = progress.add_task(
            "[cyan]Checking sites...", 
            total=min(max_check, len(potential_donation_sites))
        )
        
        for i, site in enumerate(potential_donation_sites[:max_check]):
            progress.update(task, advance=1, description=f"[cyan]Checking {i+1}/{min(max_check, len(potential_donation_sites))}: {site['url'][:50]}...")
            
            result = check_site_content(site['url'])
            
            if result['accessible'] and result['has_shopify']:
                verified_sites.append(result)
            
            time.sleep(0.5)  # Be respectful to servers
    
    return verified_sites


def display_results(sites: List[Dict]):
    """Display analysis results in a nice table"""
    
    if not sites:
        console.print("\n[red]❌ No donation sites with Shopify found[/red]")
        return
    
    console.print(f"\n[bold green]✅ Found {len(sites)} Donation Sites with Shopify Integration[/bold green]\n")
    
    # Create table
    table = Table(title="🎯 Best Donation Gates (Shopify)", box=box.DOUBLE, show_lines=True)
    table.add_column("Rank", style="cyan", width=6)
    table.add_column("URL", style="blue", width=35)
    table.add_column("Title", style="green", width=25)
    table.add_column("Shopify", style="yellow", width=8)
    table.add_column("Gateway", style="magenta", width=20)
    table.add_column("Form", style="cyan", width=6)
    table.add_column("Keywords", style="cyan", width=8)
    
    for i, site in enumerate(sites, 1):
        table.add_row(
            str(i),
            site['url'][:35],
            (site['title'][:25] if site['title'] else 'N/A'),
            "✓" if site['has_shopify'] else "✗",
            (site['payment_gateway'][:20] if site['payment_gateway'] else 'N/A'),
            "✓" if site['has_donation_form'] else "✗",
            str(site['donation_keywords_count'])
        )
    
    console.print(table)
    
    # Show payment gateways found
    console.print("\n[bold cyan]💳 Payment Gateways Detected:[/bold cyan]")
    for i, site in enumerate(sites, 1):
        if site['payment_gateway']:
            console.print(f"  {i}. {site['url']}")
            console.print(f"     Gateway: [yellow]{site['payment_gateway']}[/yellow]")
            if site['has_shopify_payments']:
                console.print(f"     Shopify Payments: [green]✓ Enabled[/green]")


def save_results(sites: List[Dict], output_file: str = 'donation_gates.json'):
    """Save results to JSON file"""
    try:
        with open(output_file, 'w') as f:
            json.dump(sites, f, indent=2)
        console.print(f"\n[green]✓ Results saved to {output_file}[/green]")
    except Exception as e:
        console.print(f"\n[red]✗ Error saving results: {e}[/red]")


def main():
    """Main function"""
    console.print("""
[bold cyan]╔═══════════════════════════════════════════════════════════╗
║         Stripeify - Donation Gate Analyzer                ║
║         Finds donation sites from Shopify gates           ║
╚═══════════════════════════════════════════════════════════╝[/bold cyan]
    """)
    
    # Load gates
    console.print("[yellow]Loading Shopify gates...[/yellow]")
    gates = load_shopify_gates('/home/null/Desktop/ShopifyGates')
    console.print(f"[green]✓ Loaded {len(gates)} Shopify gates[/green]\n")
    
    # Ask user how many to check
    console.print(f"[yellow]Total gates loaded: {len(gates)}[/yellow]")
    console.print(f"[cyan]Recommendation: Check all gates for complete database[/cyan]")
    
    try:
        max_check_input = console.input(f"[cyan]How many sites to check in detail? (Enter 'all' or number, default: all): [/]").strip().lower()
        if max_check_input == 'all' or max_check_input == '':
            max_check = len(gates)
            console.print(f"[green]✓ Will check all {len(gates)} gates[/green]")
        else:
            max_check = int(max_check_input)
            console.print(f"[green]✓ Will check {max_check} gates[/green]")
    except:
        max_check = len(gates)
        console.print(f"[green]✓ Will check all {len(gates)} gates[/green]")
    
    # Analyze gates
    verified_sites = analyze_gates(gates, max_check)
    
    # Display results
    display_results(verified_sites)
    
    # Save results
    if verified_sites:
        save_results(verified_sites, '/home/null/Desktop/Stripeify/donation_gates.json')
        
        # Ask if user wants to create a test script
        create_test = console.input("\n[cyan]Create test script for these gates? (y/n): [/]").lower()
        if create_test == 'y':
            console.print("[yellow]Creating test script...[/yellow]")
            console.print("[green]✓ Test script will be created next[/green]")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        console.print("\n\n[yellow]⚠️  Analysis interrupted by user[/yellow]")
    except Exception as e:
        console.print(f"\n\n[red]💥 Error: {e}[/red]")
