// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Stack<T> {
    head: Option<StackFrame<T>>,
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self {
            head: Default::default(),
        }
    }
}

impl<T> Clone for Stack<T> {
    fn clone(&self) -> Self {
        Self {
            head: self.head.clone(),
        }
    }
}

impl<T> Stack<T> {
    #[must_use]
    pub fn push(&self, value: T) -> Self {
        Self {
            head: Some(StackFrame::new(self.head.as_ref().cloned(), value)),
        }
    }
    #[must_use]
    pub fn pop(&self) -> Option<Self> {
        let head = self.head.as_ref()?;
        let parent = head.parent().cloned();
        Some(Self { head: parent })
    }
    #[must_use]
    pub fn peek_and_pop(&self) -> Option<(&T, Self)> {
        let existing_head = self.head.as_ref()?;
        let value = existing_head.value();
        let parent = existing_head.parent();
        Some((
            value,
            Self {
                head: parent.cloned(),
            },
        ))
    }
    #[must_use]
    pub fn append(&self, values: impl IntoIterator<Item = T>) -> Self {
        values
            .into_iter()
            .fold(self.clone(), |head, value| head.push(value))
    }
    pub fn rev(&self) -> impl Iterator<Item = &'_ T> + ExactSizeIterator + '_ {
        StackIter {
            head: self.head.as_ref(),
        }
    }
    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|head| head.value())
    }
    pub fn len(&self) -> usize {
        self.head.as_ref().map(|head| head.depth()).unwrap_or(0)
    }
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}

impl<V> FromIterator<V> for Stack<V> {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        iter.into_iter()
            .fold(Self::default(), |head, value| head.push(value))
    }
}

struct StackIter<'a, T> {
    head: Option<&'a StackFrame<T>>,
}

impl<'a, T> Clone for StackIter<'a, T> {
    fn clone(&self) -> Self {
        Self { head: self.head }
    }
}

impl<'a, T> Copy for StackIter<'a, T> {}

impl<'a, T> Iterator for StackIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.head?;
        self.head = current.parent();
        Some(current.value())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.head.map(|head| head.depth()).unwrap_or(0);
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for StackIter<'a, T> {}

#[derive(PartialEq, Eq, Hash, Debug)]
struct StackFrame<T>(std::rc::Rc<StackCell<T>>);

impl<T> StackFrame<T> {
    fn new(parent: Option<StackFrame<T>>, value: T) -> Self {
        let depth = parent.as_ref().map(|parent| parent.depth()).unwrap_or(0) + 1;
        Self(std::rc::Rc::new(StackCell {
            value,
            parent,
            depth,
        }))
    }
    fn parent(&self) -> Option<&StackFrame<T>> {
        let Self(inner) = self;
        inner.parent()
    }
    fn value(&self) -> &T {
        let Self(inner) = self;
        inner.value()
    }
    fn depth(&self) -> usize {
        let Self(inner) = self;
        inner.depth()
    }
}

impl<T> Clone for StackFrame<T> {
    fn clone(&self) -> Self {
        let Self(inner) = self;
        Self(std::rc::Rc::clone(inner))
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct StackCell<T> {
    value: T,
    parent: Option<StackFrame<T>>,
    depth: usize,
}

impl<T> StackCell<T> {
    fn parent(&self) -> Option<&StackFrame<T>> {
        self.parent.as_ref()
    }
    fn value(&self) -> &T {
        &self.value
    }
    fn depth(&self) -> usize {
        self.depth
    }
}
