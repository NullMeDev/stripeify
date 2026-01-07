from selenium import webdriver
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.chrome.service import Service

opts = Options()
opts.binary_location = "/usr/bin/chromium"
opts.add_argument("--headless=new")

svc = Service("/usr/bin/chromedriver")

driver = webdriver.Chrome(service=svc, options=opts)
driver.get("https://example.com")
print(driver.title)
driver.quit()
