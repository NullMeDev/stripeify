#!/usr/bin/env python3
"""
Quick Telegram Connection Test
Tests your Telegram bot token and chat ID independently
"""

import requests
import json

def test_telegram():
    print("=" * 60)
    print("Telegram Connection Test")
    print("=" * 60)
    
    # Load config
    try:
        with open('mady_config.json', 'r') as f:
            config = json.load(f)
        token = config['telegram_token']
        chat_id = config['chat_id']
        print(f"\n✓ Loaded config from mady_config.json")
        print(f"  Token: {token[:15]}...{token[-10:]}")
        print(f"  Chat ID: {chat_id}")
    except Exception as e:
        print(f"\n✗ Could not load config: {e}")
        print("\nPlease enter manually:")
        token = input("Bot Token: ").strip()
        chat_id = input("Chat ID: ").strip()
    
    print("\n" + "-" * 60)
    print("Step 1: Validating Token Format")
    print("-" * 60)
    
    # Validate token format
    if ':' not in token:
        print("✗ FAIL: Token should contain ':'")
        return
    
    parts = token.split(':')
    if len(parts) != 2:
        print("✗ FAIL: Token should have exactly one ':'")
        return
    
    bot_id, bot_hash = parts
    if not bot_id.isdigit():
        print("✗ FAIL: Bot ID (before ':') should be numeric")
        return
    
    print(f"✓ Token format looks valid")
    print(f"  Bot ID: {bot_id}")
    print(f"  Hash length: {len(bot_hash)} chars")
    
    print("\n" + "-" * 60)
    print("Step 2: Validating Chat ID Format")
    print("-" * 60)
    
    if chat_id.startswith('@'):
        print(f"✓ Chat ID is a username: {chat_id}")
    else:
        try:
            chat_id_num = int(chat_id)
            print(f"✓ Chat ID is numeric: {chat_id_num}")
            if chat_id_num < 0:
                print("  (Negative = Group/Channel)")
            else:
                print("  (Positive = Personal Chat)")
        except ValueError:
            print(f"✗ FAIL: Chat ID should be numeric or start with @")
            return
    
    print("\n" + "-" * 60)
    print("Step 3: Testing Bot Token with getMe")
    print("-" * 60)
    
    try:
        url = f"https://api.telegram.org/bot{token}/getMe"
        response = requests.get(url, timeout=10)
        
        print(f"Response Status: {response.status_code}")
        
        if response.status_code == 200:
            result = response.json()
            if result.get('ok'):
                bot_info = result.get('result', {})
                print(f"✓ Bot token is VALID!")
                print(f"  Bot Name: {bot_info.get('first_name')}")
                print(f"  Bot Username: @{bot_info.get('username')}")
                print(f"  Bot ID: {bot_info.get('id')}")
            else:
                print(f"✗ FAIL: {result.get('description')}")
                return
        else:
            error_data = response.json()
            print(f"✗ FAIL: HTTP {response.status_code}")
            print(f"  Error: {error_data.get('description', 'Unknown error')}")
            return
    except Exception as e:
        print(f"✗ FAIL: {e}")
        return
    
    print("\n" + "-" * 60)
    print("Step 4: Sending Test Message")
    print("-" * 60)
    
    try:
        url = f"https://api.telegram.org/bot{token}/sendMessage"
        data = {
            'chat_id': chat_id,
            'text': '🧪 TEST MESSAGE\n\n✓ Telegram connection is working!\n\nFrom: Mady Telegram Test Script',
            'parse_mode': 'HTML'
        }
        
        response = requests.post(url, data=data, timeout=10)
        
        print(f"Response Status: {response.status_code}")
        
        if response.status_code == 200:
            result = response.json()
            if result.get('ok'):
                print(f"✓ Message sent successfully!")
                print(f"  Message ID: {result.get('result', {}).get('message_id')}")
                print(f"\n{'=' * 60}")
                print("SUCCESS! Your Telegram configuration is working!")
                print("=" * 60)
            else:
                print(f"✗ FAIL: {result.get('description')}")
                print_troubleshooting(result.get('error_code'))
        else:
            error_data = response.json()
            error_code = error_data.get('error_code', response.status_code)
            error_desc = error_data.get('description', 'Unknown error')
            print(f"✗ FAIL: HTTP {response.status_code}")
            print(f"  Error Code: {error_code}")
            print(f"  Description: {error_desc}")
            
            # Check for supergroup migration
            if 'migrate_to_chat_id' in error_data.get('parameters', {}):
                new_chat_id = error_data['parameters']['migrate_to_chat_id']
                print(f"\n{'=' * 60}")
                print(f"🔄 GROUP UPGRADED TO SUPERGROUP!")
                print(f"{'=' * 60}")
                print(f"Your group was upgraded to a supergroup.")
                print(f"The chat ID has changed.\n")
                print(f"Old Chat ID: {chat_id}")
                print(f"New Chat ID: {new_chat_id}\n")
                print(f"✓ SOLUTION: Update your mady_config.json with:")
                print(f'  "chat_id": "{new_chat_id}"')
                print(f"{'=' * 60}\n")
                
                # Offer to update automatically
                try:
                    update = input("Would you like me to update mady_config.json automatically? (y/n): ").strip().lower()
                    if update == 'y':
                        import json
                        with open('mady_config.json', 'r') as f:
                            config = json.load(f)
                        config['chat_id'] = str(new_chat_id)
                        config['note'] = 'Chat ID updated - group was upgraded to supergroup'
                        with open('mady_config.json', 'w') as f:
                            json.dump(config, f, indent=2)
                        print(f"\n✓ Config updated! Run the test again to verify.")
                        return
                except:
                    pass
            else:
                print(f"\n  Full Response: {response.text[:300]}")
            
            print_troubleshooting(error_code)
    except Exception as e:
        print(f"✗ FAIL: {e}")
        return


def print_troubleshooting(error_code):
    """Print troubleshooting tips based on error code"""
    print("\n" + "-" * 60)
    print("TROUBLESHOOTING TIPS:")
    print("-" * 60)
    
    if error_code == 400:
        print("""
Common causes of Error 400 (Bad Request):

1. Bot Not Started
   → Open Telegram and send /start to your bot
   
2. Invalid Chat ID
   → Make sure the chat ID is correct
   → For groups, ID should be negative (e.g., -123456789)
   → For personal chats, ID should be positive
   
3. Bot Blocked
   → Check if you've blocked the bot
   → Unblock it in Telegram settings
   
4. Wrong Format
   → Chat ID should be a number, not a username
   → Or use @username format for public channels

How to get your Chat ID:
1. Send a message to your bot
2. Visit: https://api.telegram.org/bot<YOUR_TOKEN>/getUpdates
3. Look for "chat":{"id": YOUR_CHAT_ID}
        """)
    elif error_code == 401:
        print("""
Error 401 (Unauthorized):

Your bot token is invalid or expired.

Solution:
1. Open Telegram and search for @BotFather
2. Send /mybots
3. Select your bot
4. Click "API Token"
5. Copy the new token
        """)
    elif error_code == 403:
        print("""
Error 403 (Forbidden):

The bot cannot send messages to this chat.

Possible causes:
1. You haven't started the bot (send /start)
2. Bot is not a member of the group
3. Bot doesn't have permission to send messages
4. You've blocked the bot

Solution:
- Unblock the bot if blocked
- Add bot to group if it's a group chat
- Send /start to the bot
        """)
    else:
        print(f"""
Error {error_code}:

Check the error description above for details.
Visit: https://core.telegram.org/bots/api#making-requests
        """)


if __name__ == "__main__":
    try:
        test_telegram()
    except KeyboardInterrupt:
        print("\n\nTest interrupted by user")
    except Exception as e:
        print(f"\n\nUnexpected error: {e}")
