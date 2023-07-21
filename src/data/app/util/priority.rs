
#[derive(Debug, Clone)]
pub enum Error {
    EmptyPriority
}

pub fn choose<'a>(current: Option<&'a str>, priority: &'a [String]) -> Result<&'a str, Error> {
    if priority.len() < 1 {
        return Err(Error::EmptyPriority)
    }

    if current.is_none() {
        return Ok(priority.get(0).unwrap())
    }
    let current = current.unwrap();

    for idx in 0..priority.len() - 1 {
        let item = priority.get(idx).unwrap();
        let next_item = if let Some(next) = priority.get(idx+1) {
            next
        } else {
            priority.get(0).unwrap()
        };

        if current == item {
            return Ok(&next_item)
        }
    }

    return Ok(priority.get(0).unwrap())
}