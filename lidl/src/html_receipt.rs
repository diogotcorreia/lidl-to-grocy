use chrono::NaiveDateTime;
use ir::{Currency, Discount, ReceiptDetailed, ReceiptItem, Store};
use scraper::{node::Element, Html, Selector};

use crate::error::Error;

pub(crate) fn parse_html_receipt(
    id: String,
    date: NaiveDateTime,
    store: Store,
    html: &str,
) -> Result<ReceiptDetailed, Error> {
    let dom = Html::parse_document(html);
    let selector = Selector::parse(r#"span[id^="purchase_list_line_"]"#).unwrap();

    let mut currency = None;
    let mut items = vec![];
    let mut iter = dom.select(&selector).peekable();
    while let Some(element) = iter.next() {
        let el = element.value();
        if let Some(next_element) = iter.peek() {
            if next_element.value().id() == el.id() {
                // Lidl Germany has multiple spans with the same ID wrapping different parts of
                // the same receipt line (probably a bug on their side).
                // Skip duplicate spans if ID the same.
                continue;
            }
        }
        for class in el.classes() {
            match class {
                "currency" => {
                    let symbol = get_attr(el, "data-currency")?.to_string();
                    let id = element.text().collect::<String>().trim().to_string();
                    currency = Some(Currency { id, symbol });
                }
                "article" => {
                    // line is a sub-line if its text starts with whitespace
                    let is_article_subline = element
                        .text()
                        .next()
                        .and_then(|s| s.chars().next())
                        .map(|c| c.is_whitespace())
                        .unwrap_or(false);
                    if is_article_subline {
                        continue;
                    }

                    let id = get_attr(el, "data-art-id")?;
                    let unit_price = get_attr(el, "data-unit-price").and_then(|price| {
                        price.replace(',', ".").parse::<f64>().map_err(|_| {
                            Error::HtmlReceiptParse("cannot parse unit price as float")
                        })
                    })?;
                    let name = get_attr(el, "data-art-description")?.to_string();

                    // there isn't a is_weight field anymore, so assume it has weight if it has
                    // decimal separator; also, this attribute might be missing if it is 1
                    let (quantity, is_weight) = get_attr(el, "data-art-quantity")
                        .map(|quantity| {
                            let is_weight = quantity.contains(',');
                            let quantity =
                                quantity.replace(',', ".").parse::<f64>().map_err(|_| {
                                    Error::HtmlReceiptParse("cannot parse quantity as float")
                                })?;
                            Ok((quantity, is_weight))
                        })
                        .unwrap_or(Ok((1., false)))?;

                    let item = ReceiptItem {
                        unit_price,
                        quantity,
                        is_weight,
                        name,
                        barcode: format!("lidl-{}", id),
                        discounts: vec![],
                    };
                    items.push(item);
                }
                "discount" => {
                    let amount = element
                        .text()
                        .collect::<String>()
                        .split_whitespace()
                        .last()
                        .and_then(|amount| amount.replace(',', ".").parse::<f64>().ok())
                        .ok_or(Error::HtmlReceiptParse(
                            "cannot parse discount amount as float",
                        ))?
                        .abs();

                    if let Some(item) = items.last_mut() {
                        item.discounts.push(Discount { amount });
                    } else {
                        Err(Error::HtmlReceiptParse(
                            "found discount but there are no products before it",
                        ))?;
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(currency) = currency {
        Ok(ReceiptDetailed {
            id,
            items,
            date,
            currency,
            store,
        })
    } else {
        Err(Error::HtmlReceiptParse(
            "could not find currency in receipt",
        ))
    }
}

fn get_attr<'a>(element: &'a Element, attr: &'static str) -> Result<&'a str, Error> {
    element
        .attr(attr)
        .ok_or(Error::HtmlReceiptParseMissingAttr(attr))
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use chrono::NaiveDate;
    use ir::{Currency, Discount, ReceiptDetailed, ReceiptItem, Store};

    use super::parse_html_receipt;

    #[test]
    fn test_parse_html_receipt() -> Result<()> {
        macro_rules! s {
            ($s: expr) => {
                ($s.to_owned())
            };
        }

        let id = s!("test-id");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let store = Store {
            id: s!("STORE123"),
            name: s!("Example Store"),
        };
        let html = include_str!("../test/receipt.html");
        let receipt = parse_html_receipt(id.clone(), date, store.clone(), html)?;

        let expected = ReceiptDetailed {
            id,
            items: vec![
                ReceiptItem {
                    unit_price: 79.9,
                    quantity: 2.,
                    is_weight: false,
                    name: s!("Grytbitar"),
                    barcode: s!("lidl-0051496"),
                    discounts: vec![Discount { amount: 7.92 }],
                },
                ReceiptItem {
                    unit_price: 67.9,
                    quantity: 0.957,
                    is_weight: true,
                    name: s!("Fläskfärs 20%"),
                    barcode: s!("lidl-7006839"),
                    discounts: vec![Discount { amount: 3.22 }],
                },
                ReceiptItem {
                    unit_price: 44.9,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Lufttorkad skinka"),
                    barcode: s!("lidl-6000753"),
                    discounts: vec![Discount { amount: 2.23 }],
                },
                ReceiptItem {
                    unit_price: 42.9,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Gouda i skivor"),
                    barcode: s!("lidl-6601728"),
                    discounts: vec![Discount { amount: 2.13 }],
                },
                ReceiptItem {
                    unit_price: 36.9,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Ägg frigående"),
                    barcode: s!("lidl-7005009"),
                    discounts: vec![Discount { amount: 1.83 }],
                },
                ReceiptItem {
                    unit_price: 29.9,
                    quantity: 0.784,
                    is_weight: true,
                    name: s!("Äpple Royal Gala"),
                    barcode: s!("lidl-0081329"),
                    discounts: vec![Discount { amount: 1.16 }],
                },
                ReceiptItem {
                    unit_price: 26.9,
                    quantity: 0.814,
                    is_weight: true,
                    name: s!("Banan, EKO Fairtrade"),
                    barcode: s!("lidl-0081510"),
                    discounts: vec![Discount { amount: 1.09 }],
                },
                ReceiptItem {
                    unit_price: 19.9,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Tvättsvamp disk"),
                    barcode: s!("lidl-0155075"),
                    discounts: vec![Discount { amount: 0.99 }],
                },
                ReceiptItem {
                    unit_price: 19.9,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Friséemix"),
                    barcode: s!("lidl-7006714"),
                    discounts: vec![Discount { amount: 0.99 }],
                },
                ReceiptItem {
                    unit_price: 18.5,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Arla Mellanmjölk"),
                    barcode: s!("lidl-7003351"),
                    discounts: vec![Discount { amount: 0.92 }],
                },
                ReceiptItem {
                    unit_price: 11.9,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Zucchini styck"),
                    barcode: s!("lidl-0082346"),
                    discounts: vec![Discount { amount: 0.58 }],
                },
                ReceiptItem {
                    unit_price: 29.9,
                    quantity: 0.472,
                    is_weight: true,
                    name: s!("Sötpotatis, lösvikt"),
                    barcode: s!("lidl-0080755"),
                    discounts: vec![Discount { amount: 2.83 }, Discount { amount: 0.56 }],
                },
                ReceiptItem {
                    unit_price: 8.9,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Fruktyoghurt jord."),
                    barcode: s!("lidl-0001149"),
                    discounts: vec![Discount { amount: 0.44 }],
                },
                ReceiptItem {
                    unit_price: 8.9,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Fruktyoghurt mango"),
                    barcode: s!("lidl-0001149"),
                    discounts: vec![Discount { amount: 0.44 }],
                },
                ReceiptItem {
                    unit_price: 17.9,
                    quantity: 0.556,
                    is_weight: true,
                    name: s!("Morötter lösvikt"),
                    barcode: s!("lidl-0082755"),
                    discounts: vec![Discount { amount: 4.45 }, Discount { amount: 0.27 }],
                },
                ReceiptItem {
                    unit_price: 12.9,
                    quantity: 0.36,
                    is_weight: true,
                    name: s!("Lök, gul lösvikt"),
                    barcode: s!("lidl-0083325"),
                    discounts: vec![Discount { amount: 0.23 }],
                },
            ],
            date,
            currency: Currency {
                id: s!("SEK"),
                symbol: s!("kr"),
            },
            store,
        };

        assert_eq!(expected, receipt);

        Ok(())
    }

    #[test]
    fn test_parse_html_receipt_duplicate_spans() -> Result<()> {
        macro_rules! s {
            ($s: expr) => {
                ($s.to_owned())
            };
        }

        let id = s!("test-id");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let store = Store {
            id: s!("STORE123"),
            name: s!("Example Store"),
        };
        let html = include_str!("../test/receipt_duplicate_spans.html");
        let receipt = parse_html_receipt(id.clone(), date, store.clone(), html)?;

        let expected = ReceiptDetailed {
            id,
            items: vec![
                ReceiptItem {
                    unit_price: 2.99,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Cherrystrauchtomaten"),
                    barcode: s!("lidl-0082388"),
                    discounts: vec![Discount { amount: 1.00 }],
                },
                ReceiptItem {
                    unit_price: 1.59,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Paprika rot"),
                    barcode: s!("lidl-0082620"),
                    discounts: vec![Discount { amount: 0.48 }],
                },
                ReceiptItem {
                    unit_price: 1.79,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("K.champignons"),
                    barcode: s!("lidl-0083017"),
                    discounts: vec![],
                },
                ReceiptItem {
                    unit_price: 1.79,
                    quantity: 1.,
                    is_weight: false,
                    name: s!("Vegane Spätzle"),
                    barcode: s!("lidl-7711334"),
                    discounts: vec![],
                },
                ReceiptItem {
                    unit_price: 0.89,
                    quantity: 2.,
                    is_weight: false,
                    name: s!("Veg. Reibegenuss"),
                    barcode: s!("lidl-6612316"),
                    discounts: vec![],
                },
                ReceiptItem {
                    unit_price: 2.19,
                    quantity: 2.0,
                    is_weight: false,
                    name: s!("Bioland Tofu geräu."),
                    barcode: s!("lidl-0175011"),
                    discounts: vec![],
                },
                ReceiptItem {
                    unit_price: 0.95,
                    quantity: 1.0,
                    is_weight: false,
                    name: s!("Sojajoghurt Natur"),
                    barcode: s!("lidl-0165195"),
                    discounts: vec![],
                },
            ],
            date,
            currency: Currency {
                id: s!("EUR"),
                symbol: s!("€"),
            },
            store,
        };

        assert_eq!(expected, receipt);

        Ok(())
    }
}
