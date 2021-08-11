use std::{cell::RefCell, collections::HashMap, fs::File, path::PathBuf, rc::Rc};

use boa::{Context, Value};
use csv::StringRecord;

/// Creates the closure that will be callable in the Boa context
///
/// The returned closure takes a csv column name as first argument and returns the csv value for the current record as number
///
fn create_val_num_closure(
    record_cell: Rc<RefCell<StringRecord>>,
    name_map: HashMap<String, usize>,
) -> impl Fn(&Value, &[Value], &mut Context) -> boa::Result<Value> {
    move |_this: &Value, args: &[Value], context: &mut Context| -> boa::Result<Value> {
        // Convert mandatory first argument to string
        let var_name = args
            .get(0)
            .and_then(|val| val.as_string())
            .ok_or(context.construct_range_error("Variable name is missing"))?; // TODO convert to proper Error

        // Get the columnn index for the variable name
        let col = name_map
            .get(var_name.as_str())
            .ok_or(context.construct_range_error(format!("Cannot find Variable [{}]", var_name)))? // TODO convert to proper Error
            .to_owned();

        // Read the cell value and convert it into Boa rational value
        let record = record_cell.borrow();
        let value = if let Some(s) = record.get(col) {
            let f = s.parse::<f64>().unwrap_or(f64::NAN);
            Value::Rational(f)
        } else {
            Value::Null
        };
        Ok(value)
    }
}

fn main() -> anyhow::Result<()> {
    let path = Into::<PathBuf>::into("resources").join("iris.csv");

    let file = File::open(path)?;

    let mut reader = csv::Reader::from_reader(file);

    // Creates a map for finding csv column index from header name
    let name_map = reader
        .headers()?
        .into_iter()
        .enumerate()
        .map(|(index, name)| (name.to_owned(), index))
        .collect::<HashMap<String, usize>>();

    let record_cell = Rc::new(RefCell::new(StringRecord::new()));

    let val_num_closure = create_val_num_closure(record_cell.clone(), name_map);

    let mut context = Context::new();
    context
        .register_global_closure("val_num", 2, val_num_closure)
        .unwrap(); // TODO turn into proper Error

    // The formula to apply to all records
    let formula = r#"
        val_num('sepal.length') * val_num('sepal.width')
    "#;

    // Lets evaluate the formula for each records
    for result in reader.records() {
        let record = result?;
        record_cell.replace(record);

        let resultat = context.eval(formula).unwrap(); // TODO convert to proper Error

        println!("{:?}", resultat.as_number());
    }

    Ok(())
}
