## MODIFIED Requirements

### Requirement: Monetary values stored as TEXT in SQLite
The `Money` type's `ToSql` implementation SHALL store values as TEXT (Decimal string representation) instead of REAL (f64). The `FromSql` implementation SHALL accept both TEXT and REAL for backward compatibility with existing data.

#### Scenario: New data stored as TEXT
- **WHEN** an invoice with total_amount=1234.56 is inserted
- **THEN** the `total_amount` column in SQLite contains the TEXT value "1234.56"
- **AND** reading it back produces Money("1234.56") with full precision

#### Scenario: Old REAL data still readable
- **WHEN** the database has an old invoice with total_amount stored as REAL 1234.56
- **THEN** reading it produces Money("1234.56") without error
- **AND** the value is correct to the precision of the original f64 storage

### Requirement: Money FromSql handles both TEXT and REAL
The `Money` type's `FromSql` implementation SHALL attempt to read the value as TEXT first (Decimal string), and fall back to REAL (f64) if TEXT parsing fails. This ensures backward compatibility during the transition period.

#### Scenario: Reading TEXT-stored money
- **WHEN** a monetary column contains TEXT "99999.99"
- **THEN** `Money::from_sql` returns Money("99999.99")

#### Scenario: Reading REAL-stored money (legacy)
- **WHEN** a monetary column contains REAL 99999.99
- **THEN** `Money::from_sql` returns Money("99999.99") (via f64 conversion)
