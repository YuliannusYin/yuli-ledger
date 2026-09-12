# Preset categories

v1 **seeds** this tree on first run. After that the tree is **user data**: the UI never ships a hardcoded list of mains or subs. Create, rename, reorder, and delete (when unused) apply to mains and to their children, including seeded rows.

Preset **keys** exist only so the first-run names can go through i18n. After a rename, the screen shows the user’s name ([i18n.md](i18n.md)).

English in the `en` column is the **source UI string**. Simplified Chinese is the `zh-Hans` translation. Storage ids / keys stay English.

Related: [domain-model.md](domain-model.md) (tree rules and delete rules), [entry-kinds.md](entry-kinds.md).

## Rules

- Two UI levels: main (`parentId` null) and sub (child of a main). Entries always use a **sub**.
- Seeded mains include an **Other** sub so nothing is posted on a main node. User-created mains should get at least one sub before they can be used on an entry (the UI can add Other automatically or require the user to add a sub).
- One tree for all kinds in v1. Intended use is documented below; the app does not hide mains by kind.
- **Shopping vs Miscellaneous:** Shopping is **buying goods** (clothes, electronics, household, beauty). Miscellaneous (**消费**) is the **catch-all** for outflows that do not belong in Transport, Food, Housing, Entertainment, Telecom, Education, Investment, Shopping, or Medical. Do not put groceries in Miscellaneous (that is Food) or clothes in Miscellaneous (that is Shopping).
- Kind **`transfer`** and main category **Transfer** are different things. The kind moves money between accounts. The category classifies that move (WeChat withdrawal vs internal shift). Transfer **fee** defaults to Transfer → Transfer fee, not Finance.

## User-owned tree (not compiled in)

- Pickers load mains and subs from SQLite.
- The seed in this file is **initial data**, not a closed enum in the client.
- Do not `match` on `preset.category.transport` in UI logic. The only stored special-case id is `LedgerSettings.defaultFeeCategoryId`, which starts as the seeded Transfer-fee sub and can be pointed at any other sub (or cleared) if the user deletes or replaces it.
- Seed `colorHex` on each main from the table in [ui.md](ui.md). Subs have no stored color. The user may later pick another palette color for a main.

## Investment (read this)

`investment` is a **category**, not an entry kind. If you post a fund purchase as `expense` + Investment, v1 **P&L treats it as spending**. Moving cash into a brokerage **account** should be a `transfer`. True investing (positions, cost basis) is out of v1.

## Seeded tree

`sortOrder` follows this table (mains, then children). Keys: `preset.category.<main>` and `preset.category.<main>.<sub>`.

### Transport · 交通

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.transport` | Transport | 交通 |
| `preset.category.transport.bus` | Bus | 公交 |
| `preset.category.transport.metro` | Metro | 地铁 |
| `preset.category.transport.taxi` | Taxi | 打车 |
| `preset.category.transport.train` | Train | 列车 |
| `preset.category.transport.flight` | Flight | 飞机 |
| `preset.category.transport.other` | Other | 其他 |

### Food · 餐饮

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.food` | Food | 餐饮 |
| `preset.category.food.dining` | Dining out | 堂食 |
| `preset.category.food.takeout` | Takeout | 外卖 |
| `preset.category.food.groceries` | Groceries | 食材 |
| `preset.category.food.cafe` | Drinks | 饮品 |
| `preset.category.food.other` | Other | 其他 |

### Housing · 住房

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.housing` | Housing | 住房 |
| `preset.category.housing.rent` | Rent | 租金 |
| `preset.category.housing.utilities` | Utilities | 水电燃气 |
| `preset.category.housing.property` | Property fee | 物业 |
| `preset.category.housing.furnishings` | Furnishings | 家居 |
| `preset.category.housing.maintenance` | Maintenance | 维修 |
| `preset.category.housing.other` | Other | 其他 |

Advance rent: kind **`prepayment`** on the account whose **debt** should rise (often a dedicated account), category Housing → Rent. If cash also left a wallet, record that as a separate `expense` or `transfer`.

### Entertainment · 娱乐

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.entertainment` | Entertainment | 娱乐 |
| `preset.category.entertainment.games` | Games | 游戏 |
| `preset.category.entertainment.streaming` | Streaming | 流媒体 |
| `preset.category.entertainment.events` | Events | 影视演出 |
| `preset.category.entertainment.outing` | Outing | 出游 |
| `preset.category.entertainment.sports` | Sports | 运动 |
| `preset.category.entertainment.other` | Other | 其他 |

### Telecom · 通讯

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.telecom` | Telecom | 通讯 |
| `preset.category.telecom.mobile` | Mobile | 话费流量 |
| `preset.category.telecom.broadband` | Broadband | 宽带 |
| `preset.category.telecom.software` | Software | 软件订阅 |
| `preset.category.telecom.other` | Other | 其他 |

### Education · 教育

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.education` | Education | 教育 |
| `preset.category.education.tuition` | Tuition | 学费 |
| `preset.category.education.books` | Books | 书本 |
| `preset.category.education.courses` | Courses | 课程培训 |
| `preset.category.education.other` | Other | 其他 |

### Investment · 投资

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.investment` | Investment | 投资 |
| `preset.category.investment.funds` | Funds | 基金 |
| `preset.category.investment.stocks` | Stocks | 股票 |
| `preset.category.investment.wealth` | Wealth management | 理财 |
| `preset.category.investment.other` | Other | 其他 |

### Shopping · 购物

Buying **goods**. Not meals (Food), not tickets (Transport / Entertainment).

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.shopping` | Shopping | 购物 |
| `preset.category.shopping.clothing` | Clothing | 服饰 |
| `preset.category.shopping.electronics` | Electronics | 数码 |
| `preset.category.shopping.household` | Household | 日用 |
| `preset.category.shopping.beauty` | Beauty | 美妆 |
| `preset.category.shopping.other` | Other | 其他 |

### Miscellaneous · 消费

Catch-all only. If a more specific main fits, use that.

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.misc` | Miscellaneous | 消费 |
| `preset.category.misc.gifts` | Gifts | 礼金礼物 |
| `preset.category.misc.services` | Services | 服务 |
| `preset.category.misc.other` | Other | 其他 |

### Medical · 医疗

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.medical` | Medical | 医疗 |
| `preset.category.medical.outpatient` | Outpatient | 就诊 |
| `preset.category.medical.medicine` | Medicine | 药品 |
| `preset.category.medical.insurance` | Insurance | 医疗保险 |
| `preset.category.medical.other` | Other | 其他 |

### Income · 收入

Intended for kind `income`. English source name is **Income**, not a calque of 工资 as the main.

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.income` | Income | 收入 |
| `preset.category.income.salary` | Salary | 工资 |
| `preset.category.income.bonus` | Bonus | 奖金 |
| `preset.category.income.sidejob` | Side job | 兼职 |
| `preset.category.income.gift` | Gift | 红包礼金 |
| `preset.category.income.refund` | Refund | 退款 |
| `preset.category.income.other` | Other | 其他 |

### Transfer · 转账

Classifies **`transfer` entries** (the kind). Not a second entry kind. Distinct from **Finance**.

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.transfer` | Transfer | 转账 |
| `preset.category.transfer.wechat` | WeChat withdrawal | 微信提现 |
| `preset.category.transfer.internal` | Between accounts | 账户互转 |
| `preset.category.transfer.fee` | Transfer fee | 转账手续费 |
| `preset.category.transfer.other` | Other | 其他 |

- Transfer `categoryId`: e.g. WeChat withdrawal / Between accounts (required like other kinds).
- Transfer `feeCategoryId` when fee > 0: defaults to Transfer fee (`preset.category.transfer.fee` at seed; then `defaultFeeCategoryId`).

### Finance · 财务

Bank charges and a home for `repayment` category when the payment is not Housing (or similar). Transfer **fees** live under Transfer, not here.

| Key | en | zh-Hans |
|-----|----|---------|
| `preset.category.finance` | Finance | 财务 |
| `preset.category.finance.bankfee` | Bank fee | 银行费用 |
| `preset.category.finance.repayment` | Repayment | 还款 |
| `preset.category.finance.other` | Other | 其他 |

## Changing the tree later

- **Add** a main or a sub: user-created rows have `presetKey` null.
- **Rename**: write `name`; stop using the i18n catalog for that row.
- **Reorder**: `sortOrder`.
- **Delete** a sub: allowed only if no entry uses it as `categoryId` or `feeCategoryId`, and it is not the current `defaultFeeCategoryId` (clear or retarget that setting first).
- **Delete** a main: allowed only if every descendant can be deleted (same rules); delete the children in the same operation.
- Occupied nodes: **refuse**. Reassign or delete those entries first. v1 has no bulk reassign dialog.
- A future app version must not delete occupied rows from old databases. Do not reuse a retired preset key for a different meaning.
