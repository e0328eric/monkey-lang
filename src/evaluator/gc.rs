// This code is very unsafe but I realize it after implementing all of the evaluating part.
// GCBox<T> can be freed only if it is linked into GC<T>.

use crate::object::{EnvWrapper, Object};
use std::cell::Cell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct GCBox<T> {
    is_marked: bool,
    inner: Box<T>,
    next: Cell<Option<*mut GCBox<T>>>,
    _marker: PhantomData<GCBox<T>>,
}

impl<T> Deref for GCBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T> DerefMut for GCBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

impl<T: Clone> Clone for GCBox<T> {
    fn clone(&self) -> Self {
        let output = Self {
            is_marked: self.is_marked,
            inner: self.inner.clone(),
            next: self.next.clone(),
            _marker: PhantomData,
        };
        self.next.set(None);
        output
    }
}

impl<T: PartialEq> PartialEq for GCBox<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T> GCBox<T> {
    pub fn new(source: T) -> Self {
        Self {
            is_marked: false,
            inner: Box::new(source),
            next: Cell::new(None),
            _marker: PhantomData,
        }
    }

    fn link_next(&mut self, other: *mut Self) {
        self.next.set(Some(other));
    }

    pub fn mark(&mut self) {
        self.is_marked = true;
    }
}

impl GCBox<Object> {
    pub fn mark_env(env: &EnvWrapper) {
        for e in env.as_ref().borrow_mut().store.iter_mut() {
            e.1.mark();
        }
        if let Some(outer_env) = &env.as_ref().borrow().outer {
            GCBox::mark_env(&outer_env);
        }
    }
}

pub struct GC<T> {
    pub head: GCBox<T>,
}

impl<T> GC<T> {
    // This function is actually unsafe because to_add can be removed
    // before reading it in GC
    pub fn add(&mut self, to_add: &mut GCBox<T>) {
        self.head.link_next(to_add as *mut GCBox<T>);
    }

    pub fn sweep(&mut self) {
        let mut node = &mut self.head;
        while node.next.get().is_some() {
            // SAFETY: We are making a null pointer if node.next is Some.
            // So, dereferensing node.next is fine.
            unsafe {
                if (*node.next.get().unwrap()).is_marked {
                    let tmp = node.next.get().unwrap();
                    node.next.set((*tmp).next.get());
                    drop(Box::from_raw(tmp));
                } else {
                    node = &mut *node.next.get().unwrap();
                }
            }
        }
        node = &mut self.head;
        while node.next.get().is_some() {
            // SAFETY: We are making a null pointer if node.next is Some.
            // So, dereferensing node.next is fine.
            unsafe {
                (*node.next.get().unwrap()).is_marked = false;
                node = &mut *node.next.get().unwrap();
            }
        }
    }
}
