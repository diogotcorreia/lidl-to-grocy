# Lidl to Grocy

Import receipts from Lidl Plus to Grocy, right from your terminal!

[![asciicast](https://asciinema.org/a/636413.svg)](https://asciinema.org/a/636413)

## Why?

Why not?

## How?

When first run, the program asks you to login into your Lidl account (through OAuth2)
and to your Grocy instance.
Then, it fetches the latest receipts from Lidl and allows you to pick one of them to
start the import process.

Purchase date, price, store, quantities, and discounts are all accounted for and
imported properly into Grocy. Unfortunately, it is not possible to get due date
information from the receipt, so that is prompted to the user.

## Features

- Save credentials and store mappings between runs
- Import both quantity and weight products
- Insert due dates per product (even if multiple of the same product were purchased)
- Subtract discounts from the product price
- Associate barcode with product if it does not exist already
- Skip importing products
- Respect default due dates and locations from Grocy

## Configuration

This program stores its configuration in:
- `$XDG_CONFIG_HOME/lidl-to-grocy/lidl-to-grocy.toml` on Linux
- `$HOME/Library/Application Support/lidl-to-grocy/lidl-to-grocy.toml` on MacOS
- `{FOLDERID_RoamingAppData}\lidl-to-grocy\lidl-to-grocy.toml` on Windows

You'll likely never need to edit the configuration by hand, as the program prompts
you for configuration the first time you run it or in case any value is missing.

## Contributions

If you find a bug in this program or want to add some new feature, please open an issue
on this repository and/or a pull request!

## Disclaimer

This program makes unofficial use of the Lidl Plus API and might stop
working at any time.
It is, in no way, shape or form, associated with neither Lidl nor Grocy.
