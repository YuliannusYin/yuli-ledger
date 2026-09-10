use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::Result;
use crate::time_util::OPENING_EPOCH;

struct SeedMain {
    key: &'static str,
    color: &'static str,
    subs: &'static [&'static str],
}

const MAINS: &[SeedMain] = &[
    SeedMain {
        key: "preset.category.transport",
        color: "#5b7c99",
        subs: &[
            "preset.category.transport.bus",
            "preset.category.transport.metro",
            "preset.category.transport.taxi",
            "preset.category.transport.train",
            "preset.category.transport.flight",
            "preset.category.transport.other",
        ],
    },
    SeedMain {
        key: "preset.category.food",
        color: "#b4532a",
        subs: &[
            "preset.category.food.dining",
            "preset.category.food.takeout",
            "preset.category.food.groceries",
            "preset.category.food.cafe",
            "preset.category.food.other",
        ],
    },
    SeedMain {
        key: "preset.category.housing",
        color: "#78716c",
        subs: &[
            "preset.category.housing.rent",
            "preset.category.housing.utilities",
            "preset.category.housing.property",
            "preset.category.housing.furnishings",
            "preset.category.housing.maintenance",
            "preset.category.housing.other",
        ],
    },
    SeedMain {
        key: "preset.category.entertainment",
        color: "#a16207",
        subs: &[
            "preset.category.entertainment.games",
            "preset.category.entertainment.streaming",
            "preset.category.entertainment.events",
            "preset.category.entertainment.outing",
            "preset.category.entertainment.sports",
            "preset.category.entertainment.other",
        ],
    },
    SeedMain {
        key: "preset.category.telecom",
        color: "#0e7490",
        subs: &[
            "preset.category.telecom.mobile",
            "preset.category.telecom.broadband",
            "preset.category.telecom.software",
            "preset.category.telecom.other",
        ],
    },
    SeedMain {
        key: "preset.category.education",
        color: "#4f46e5",
        subs: &[
            "preset.category.education.tuition",
            "preset.category.education.books",
            "preset.category.education.courses",
            "preset.category.education.other",
        ],
    },
    SeedMain {
        key: "preset.category.investment",
        color: "#3f6212",
        subs: &[
            "preset.category.investment.funds",
            "preset.category.investment.stocks",
            "preset.category.investment.wealth",
            "preset.category.investment.other",
        ],
    },
    SeedMain {
        key: "preset.category.shopping",
        color: "#9f1239",
        subs: &[
            "preset.category.shopping.clothing",
            "preset.category.shopping.electronics",
            "preset.category.shopping.household",
            "preset.category.shopping.beauty",
            "preset.category.shopping.other",
        ],
    },
    SeedMain {
        key: "preset.category.misc",
        color: "#52525b",
        subs: &[
            "preset.category.misc.gifts",
            "preset.category.misc.services",
            "preset.category.misc.other",
        ],
    },
    SeedMain {
        key: "preset.category.medical",
        color: "#be185d",
        subs: &[
            "preset.category.medical.outpatient",
            "preset.category.medical.medicine",
            "preset.category.medical.insurance",
            "preset.category.medical.other",
        ],
    },
    SeedMain {
        key: "preset.category.income",
        color: "#0f766e",
        subs: &[
            "preset.category.income.salary",
            "preset.category.income.bonus",
            "preset.category.income.sidejob",
            "preset.category.income.gift",
            "preset.category.income.refund",
            "preset.category.income.other",
        ],
    },
    SeedMain {
        key: "preset.category.transfer",
        color: "#57534e",
        subs: &[
            "preset.category.transfer.wechat",
            "preset.category.transfer.internal",
            "preset.category.transfer.fee",
            "preset.category.transfer.other",
        ],
    },
    SeedMain {
        key: "preset.category.finance",
        color: "#1e3a5f",
        subs: &[
            "preset.category.finance.bankfee",
            "preset.category.finance.repayment",
            "preset.category.finance.other",
        ],
    },
];

pub fn seed_if_empty(conn: &Connection) -> Result<()> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM ledger_settings", [], |r| r.get(0))?;
    if n > 0 {
        return Ok(());
    }
    let tx = conn.unchecked_transaction()?;
    let account_id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO account (id, name, account_kind, opening_balance_minor, opening_debt_minor, opening_at, note, sort_order, preset_key)
         VALUES (?1, NULL, 'other', 0, 0, ?2, NULL, 0, 'preset.account.default')",
        params![account_id, OPENING_EPOCH],
    )?;

    let mut fee_id: Option<String> = None;
    for (mi, main) in MAINS.iter().enumerate() {
        let main_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO category (id, parent_id, name, preset_key, sort_order, color_hex)
             VALUES (?1, NULL, NULL, ?2, ?3, ?4)",
            params![main_id, main.key, mi as i32, main.color],
        )?;
        for (si, sub) in main.subs.iter().enumerate() {
            let sub_id = Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO category (id, parent_id, name, preset_key, sort_order, color_hex)
                 VALUES (?1, ?2, NULL, ?3, ?4, NULL)",
                params![sub_id, main_id, sub, si as i32],
            )?;
            if *sub == "preset.category.transfer.fee" {
                fee_id = Some(sub_id);
            }
        }
    }

    tx.execute(
        "INSERT INTO ledger_settings (
            id, currency_code, default_account_id, schema_version,
            ui_language, color_scheme, default_fee_category_id,
            report_mode, report_side, report_custom_from, report_custom_to
         ) VALUES (1, 'CNY', ?1, 2, NULL, NULL, ?2, NULL, NULL, NULL, NULL)",
        params![account_id, fee_id],
    )?;
    tx.commit()?;
    Ok(())
}
