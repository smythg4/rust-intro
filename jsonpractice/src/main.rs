use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
enum Gender {
    Male,
    Female,
}

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    name: String,
    age: u8,
    gender: Gender,
}

impl Person {
    fn new(name: &str, age: u8, gender: Gender) -> Self {
        Person {
            name: name.to_string(),
            age,
            gender,
        }
    }

    fn from_json(filepath: &str) -> Result<Person> {
        let json_data = fs::read_to_string(filepath)
            .map_err(serde_json::Error::io)?;
        let person: Person = serde_json::from_str(&json_data)?;

        Ok(person)
    }

    fn to_file(&self, filepath: &str) -> Result<()> {
        let json_data = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(filepath)
            .map_err(serde_json::Error::io)?;

        file.write_all(json_data.as_bytes())
            .map_err(serde_json::Error::io)?;

        Ok(())
    }

    fn load_people_from_json(filepath: &str) -> Result<Vec<Person>> {
        let json_data = fs::read_to_string(filepath)
            .map_err(serde_json::Error::io)?;
        let people: Vec<Person> = serde_json::from_str(&json_data)?;

        Ok(people)
    }

    fn save_people_to_file(people: &[Person], filepath: &str) -> Result<()> {
        let json_data = serde_json::to_string_pretty(people)?;
        let mut file = File::create(filepath)
            .map_err(serde_json::Error::io)?;

        file.write_all(json_data.as_bytes())
            .map_err(serde_json::Error::io)?;
        Ok(())
    }
}
fn main() -> Result<()> {
    let filepath = "people.json";
    let people = Person::load_people_from_json(filepath)?;

    for person in &people {
        println!("Person: {:?}", person);
    }

    let mut all_people = people;
    all_people.push(Person::new("David", 28, Gender::Male));

    Person::save_people_to_file(&all_people, filepath)?;

    Ok(())
}
