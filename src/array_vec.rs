use core::mem::MaybeUninit;

pub struct ArrayVec<T, const N: usize> {
    pub length: usize,
    pub items: [MaybeUninit<T>; N],
}

impl<T, const N: usize> ArrayVec<T, N> {
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.length == N {
            return Err(value);
        }
        self.items[self.length].write(value);
        self.length += 1;
        return Ok(());
    }

    //Get non-mutable refrences -> &T
    pub fn iter(&self) -> ArrayVecIter<T, N> {
        ArrayVecIter {
            array_vec: self,
            index: 0,
        }
    }

    //Get mutable refrences ->&mut T
    pub fn iter_mut(&mut self) -> ArrayVecIterMut<T, N> {
        ArrayVecIterMut {
            array_vec: self,
            index: 0,
        }
    }
}

pub struct ArrayVecIter<'a, T, const N: usize> {
    array_vec: &'a ArrayVec<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for ArrayVecIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.array_vec.length {
            None
        } else {
            let item = Some(unsafe { self.array_vec.items[self.index].assume_init_ref() });
            self.index += 1;
            item
        }
    }
}

pub struct ArrayVecIterMut<'a, T, const N: usize> {
    array_vec: &'a mut ArrayVec<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for ArrayVecIterMut<'a, T, N> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.array_vec.length {
            None
        } else {
            let item = Some(unsafe {
                &mut *(self.array_vec.items[self.index].assume_init_mut() as *mut T)
            });
            self.index += 1;
            item
        }
    }
}
