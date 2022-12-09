// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{FloatTermType, FloatValue};

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FloatTerm {
    value: [u32; 2],
}
impl TermSize for FloatTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for FloatTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.value, arena)
    }
}
impl From<f64> for FloatTerm {
    fn from(value: f64) -> Self {
        Self {
            value: f64_to_chunks(value),
        }
    }
}
impl Into<f64> for FloatTerm {
    fn into(self) -> f64 {
        let Self { value, .. } = self;
        chunks_to_f64(value)
    }
}

fn f64_to_chunks(value: f64) -> [u32; 2] {
    let bytes = value.to_le_bytes();
    let low_word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let high_word = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    [low_word, high_word]
}

fn chunks_to_f64(value: [u32; 2]) -> f64 {
    let [low_word, high_word] = value;
    let low_bytes = low_word.to_le_bytes();
    let high_bytes = high_word.to_le_bytes();
    f64::from_le_bytes([
        low_bytes[0],
        low_bytes[1],
        low_bytes[2],
        low_bytes[3],
        high_bytes[0],
        high_bytes[1],
        high_bytes[2],
        high_bytes[3],
    ])
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, FloatTerm, A> {
    fn value(&self) -> f64 {
        chunks_to_f64(self.as_deref().value)
    }
}

impl<'heap, A: ArenaAllocator> FloatTermType for ArenaRef<'heap, FloatTerm, A> {
    fn value(&self) -> FloatValue {
        self.value()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn float() {
        let value = 3.142;
        assert_eq!(
            TermType::Float(FloatTerm::from(value)).as_bytes(),
            [
                TermTypeDiscriminants::Float as u32,
                f64_to_chunks(value)[0],
                f64_to_chunks(value)[1]
            ],
        );
    }
}
