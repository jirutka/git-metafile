use std::result::Result;

pub struct InspectErr<I, F> {
    iter: I,
    f: F,
}

impl<I, F, T, E> Iterator for InspectErr<I, F>
where
    I: Iterator<Item = Result<T, E>>,
    F: FnMut(&E),
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        self.iter.next().map(|next| {
            if let Err(ref e) = next {
                (self.f)(e);
            }
            next
        })
    }
}

pub trait IteratorExt: Iterator {
    fn inspect_err<T, E, F>(self, f: F) -> InspectErr<Self, F>
    where
        Self: Iterator<Item = Result<T, E>> + Sized,
        F: FnMut(&E),
    {
        InspectErr { iter: self, f }
    }
}

impl<I: ?Sized> IteratorExt for I where I: Iterator {}

#[cfg(test)]
mod tests {
    use super::IteratorExt;

    #[test]
    fn inspect_err() {
        let input = vec![Ok(0), Err("err1"), Ok(2), Err("err3")];
        let mut inspected = vec![];

        let output = input
            .clone()
            .into_iter()
            .inspect_err(|&e| inspected.push(e))
            .collect::<Vec<_>>();

        // Does not modify the iterated vec.
        assert_eq!(&output[..], &input[..]);

        // Inspects Err values.
        assert_eq!(&inspected[..], ["err1", "err3"]);
    }
}
