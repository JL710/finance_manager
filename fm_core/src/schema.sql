CREATE TABLE IF NOT EXISTS asset_account (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    notes TEXT,
    iban TEXT,
    bic TEXT
);

CREATE TABLE IF NOT EXISTS book_checking_account (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    notes TEXT,
    iban TEXT,
    bic TEXT
);

CREATE TABLE IF NOT EXISTS account (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    asset_account INTEGER,
    book_checking_account INTEGER,
    FOREIGN KEY(asset_account) REFERENCES asset_account(id),
    FOREIGN KEY (book_checking_account) REFERENCES book_checking_account(id)
);

CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    amount_value INTEGER NOT NULL,
    currency INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    source_id INTEGER NOT NULL,
    destination_id INTEGER NOT NULL,
    budget INTEGER,
    timestamp INTEGER,
    FOREIGN KEY(source_id) REFERENCES account(id),
    FOREIGN KEY(destination_id) REFERENCES account(id),
    FOREIGN KEY (budget) REFERENCES budget(id)
);

CREATE TABLE IF NOT EXISTS budget (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    value INTEGER NOT NULL,
    currency INTEGER NOT NULL,
    timespan_type INTEGER NOT NULL,
    timespan_field1 INTEGER NOT NULL,
    timespan_field2 INTEGER
);