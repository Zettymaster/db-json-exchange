use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};

/// The root structure, containing the rough layout of databases,
/// as well as the transactions to run on them.
///
/// When the [Database] is incompatible with the database system or colums are mismatched,
/// an implementation MUST disregard ALL transactions,
/// not just the ones that reference incompatible databases/tables.
#[derive(Serialize, Deserialize, Debug)]
pub struct DefinitionFile{
    /// The databases that are expected to exist for the transactions to function
    pub databases: Vec<Database>,
    /// The transactions to run, in order
    pub transactions: Vec<Transaction>
}

/// A database that is expected to exist and its tables.
///
/// Note that in [Transaction] the name is referenced via a [DatabaseReference]
#[derive(Serialize, Deserialize, Debug)]
pub struct Database{
    /// The unique identifier of the database.
    /// A database is later referenced by this name using a [DatabaseReference] in [Transaction].
    /// A database name may only contain:
    ///  - the letters a-z and A-Z
    ///  - the numbers 0-9
    ///  - the symbols '_', '-', '.'
    pub name: String,
    /// the tables that are expected to exist
    pub tables: Vec<Table>
}

/// A Table inside a database.
/// Note that columns are referenced by their name later on, not their ordinal.
///
/// Can be referenced in an [Statement] using a [TableReference]
#[derive(Serialize, Deserialize, Debug)]
pub struct Table{
    /// the unique identifier of the table.
    /// Unique in its Database (2 databases may have tables with equal names)
    /// A table name may only contain:
    //   - the letters a-z and A-Z
    //   - the numbers 0-9
    //   - the symbols '_', '-', '.'
    pub name: String,
    /// the columns in arbitrary order (see note)
    pub columns: Vec<Column>
}

/// A column definition with its name and type.
/// For conversions and notes on compatibility, see [Type].
#[derive(Serialize, Deserialize, Debug)]
pub struct Column{
    /// the unique identifier of the column.
    /// You can have 2 Id columns as long as they are in different tables.
    ///
    /// A table name may only contain:
    ///  - the letters a-z and A-Z
    ///  - the numbers 0-9
    ///  - the symbols '_', '-', '.'
    pub name: String,
    /// the type of the column
    #[serde(rename = "type", flatten)]
    pub _type: Type
}

/// The Type of Column and requirements that are needed to satisfy basic requirements.
///
/// Care should be taken to verify that [Expression]s only assume valid conversions,
/// but since the rules for converting and even the presence of a given type is dependent
/// on the actual database used, they cannot be set in stone here.
///
/// Compatibility to your database's built in types MUST be checked, before attempting to execute [Transaction]s.
#[derive(Serialize, Deserialize, Debug)]
pub struct Type{
    /// the actual data type
    pub data_type: DataType,
    /// if values of a given column must be unique
    pub unique: bool,
    /// if values may be null
    pub not_null: bool
}

/// the actual inner type of column
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "bounds")]
pub enum DataType{
    /// A boolean, with values true and false.
    ///
    /// Do note, that ONLY true and false are specified,
    /// conversions to/from integers are optional and implementation defined!
    Bool,
    /// any length of whole number
    Int{ upper: isize, lower: isize },
    /// any length of decimal number
    Double{ upper: f64, lower: f64 },
    /// A string of characters.
    ///
    /// Do note that while everything in this format is encoded in UTF-8 No BOM,
    /// you may need to convert a string your native encoding.
    /// Care should be taken to properly escape Strings as they run the risk of SQL injection.
    String{ min_chars: usize, max_chars: usize }
}

/// A list of [Statements] to run on a single given database.
///
/// While it is implementation defined, if a Transaction is run inside of an actual
/// transaction on the database, it is HIGHLY encouraged and
/// not doing so should at the very least be logged.
/// It is REQUIRED to skip statements that would occur after one that failed (and roll back the transaction, if supported).
/// As an example with 4 statements, if statement 1 executes fine, but 2 throws an error, 3 and 4 MUST be skipped.
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction{
    /// the database to run statements on.
    pub database: DatabaseReference,
    /// the statements to run in order.
    pub statements: Vec<Statement>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseReference{
    pub database: String
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Statement {
    NoOp(NoOp),
    Select(Select),
    Delete(Delete),
    Insert(Insert),
    Update(Update)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NoOp{
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Select {
    pub table: TableReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_condition: Option<Expression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Expression>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Delete{
    pub table: TableReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_condition: Option<Expression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Expression>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Insert{
    pub table: TableReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_condition: Option<Expression>,
    pub data: Vec<Map<String, Value>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Update{
    pub table: TableReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_condition: Option<Expression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Expression>,
    pub data: Map<String, Value>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableReference{
    pub table: String
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Expression{
    Constant(Constant),
    ColumnReference(ColumnReference),
    Not(Box<Expression>),
    NotNull{casted: Box<Expression>, default: Box<Expression>},
    Equals{left: Box<Expression>, right: Box<Expression>},
    LessThan{left: Box<Expression>, right: Box<Expression>},
    GreaterThan{left: Box<Expression>, right: Box<Expression>},
    LessThanOrEqual{left: Box<Expression>, right: Box<Expression>},
    GreaterThanOrEqual{left: Box<Expression>, right: Box<Expression>},
    Plus{left: Box<Expression>, right: Box<Expression>},
    Subtract{left: Box<Expression>, right: Box<Expression>},
    Divide{left: Box<Expression>, right: Box<Expression>},
    Multiply{left: Box<Expression>, right: Box<Expression>},
    Modulo{left: Box<Expression>, right: Box<Expression>},
    And{left: Box<Expression>, right: Box<Expression>},
    Or{left: Box<Expression>, right: Box<Expression>},
    BitAnd{left: Box<Expression>, right: Box<Expression>},
    BitOr{left: Box<Expression>, right: Box<Expression>},
    BitXOr{left: Box<Expression>, right: Box<Expression>},
    Contains{left: Box<Expression>, right: Box<Expression>},
    StartsWith{left: Box<Expression>, right: Box<Expression>},
    EndsWith{left: Box<Expression>, right: Box<Expression>},
    Conditional{condition: Box<Expression>, true_path: Box<Expression>, false_path: Box<Expression>}
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Constant{
    Bool(bool),
    Int(usize),
    Double(f64),
    String(String)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ColumnReference {
    #[serde(flatten)]
    pub table: TableReference,
    pub column: String
}