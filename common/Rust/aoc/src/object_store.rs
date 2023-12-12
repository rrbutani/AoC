use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::{borrow::Borrow, collections::HashSet};

// We can implement Hash and Eq here because [`ObjectStore`] ensures that each
// index (the `usize` here) corresponds to a unique instance.
#[derive(Debug)]
pub struct Ref<'store, T: Eq + Hash, UniqueMarker = ()>(
    usize,
    PhantomData<(&'store mut (), T, UniqueMarker)>,
);

impl<'s, T: Eq + Hash, U> Copy for Ref<'s, T, U> {}
impl<'s, T: Eq + Hash, U> Clone for Ref<'s, T, U> {
    fn clone(&self) -> Self {
        Ref(self.0, PhantomData)
    }
}

impl<'s, T: Eq + Hash, U> Eq for Ref<'s, T, U> {}
impl<'s, T: Eq + Hash, U> PartialEq for Ref<'s, T, U> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'s, T: Eq + Hash, U> Hash for Ref<'s, T, U> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

/// A private type that only exists so we can lookup and map values to
/// references on the [`HashSet`] inside of [`ObjectStore`].
//
// Note: This is wasteful since every instance will have the same reference to
// the same Vec but I can't come up with a way to sneak in the Vec so that the
// Hash and Eq impls can use it that's better than this.
#[derive(Debug)]
struct InnerRef<'store, T: Eq + Hash, UniqueMarker = ()>(
    *const Vec<T>,
    Ref<'store, T, UniqueMarker>,
);

/// Note: easy to alias things in illegal ways using this; ObjectStore must take
/// care not to.
impl<'store, T: Eq + Hash, U> Borrow<T> for InnerRef<'store, T, U> {
    fn borrow(&self) -> &T {
        #[allow(unsafe_code)]
        let v = unsafe { &*self.0 };
        &v[self.1 .0]
    }
}

impl<'store, T: Eq + Hash, U> Hash for InnerRef<'store, T, U> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Safety: these references cannot outlive the `Vec` they reference
        // (the lifetime should ensure this) and `ObjectStore` must make sure
        // that the data this inner pointer refers to is not deferenced while
        // there is a mutable borrow out on the data.
        #[allow(unsafe_code)]
        let v = unsafe { &*self.0 };
        Hash::hash(&v[self.1 .0], state)
    }
}

impl<'store, T: Eq + Hash, U> Eq for InnerRef<'store, T, U> {}
impl<'store, T: Eq + Hash, U> PartialEq for InnerRef<'store, T, U> {
    // Since the object store is supposed to make sure that no two things inside
    // of the inner Vec are the same, we _could_ just compare the indexes.
    fn eq(&self, other: &Self) -> bool {
        #[allow(unsafe_code)]
        let v = unsafe { &*self.0 };

        PartialEq::eq(&v[self.1 .0], &v[other.1 .0])
    }
}

impl<'store, T: Eq + Hash, U> PartialEq<T> for InnerRef<'store, T, U> {
    fn eq(&self, other: &T) -> bool {
        #[allow(unsafe_code)]
        let v = unsafe { &*self.0 };

        PartialEq::eq(&v[self.1 .0], other)
    }
}

// TODO: Instead of doing the below we could store wrapped raw pointers in a
// HashSet and also use a Vec of Vecs to ensure that we never move (and thus
// invalidate pointers to) any already stored objects.
//
// Refs could also just store raw pointers with this scheme though we'd still
// probably want to have them be passed to their source ObjectStore? We don't
// actually need to enforce mutability ^ aliasing across all objects from the
// object store actually so maybe we don't need to do this...
//
// It's worth noting that the footgun that `UniqueMarker` exists to guard
// against (described and demonstrated in the docs below) would be eliminated
// under this scheme: since the Refs contain a raw pointer they don't use the
// state in their ObjectStore at all when obtaining a reference.
//
// Additionally we can switch to using a HashSet under this scheme because
// there's no mapping involved; we're storing raw pointers (albeit, wrapped) in
// the HashSet and Refs also just contain raw pointers.
//
// Some misc things:
//   - we'd probably want to make progressively larger Vecs as we need to make
//     new ones; probably powers of two or something
//   - on Drop we might want to empty the initial Vec object store? this goes
//     for the current impl too
//   - not sure how mutability will work and that's the reason why I'm not
//     rewriting the below to be like this yet
//       * we can't just have Ref's have a get_mut method because we *do* want
//         to make sure that we don't alias mutably multiple times/violate
//         mutability ^ aliasing
//       * maybe we can have those methods return references that are bounded
//         by the lifetime of the borrow of the Ref instance? we'd need to make
//         Ref explicitly *not* Clone or Copy
//           + this gets tricky because we'd also have to not provide lookup
//             functionality on the ObjectStore then which is probably fine for
//             our specific use case, actually
//           + but it does mean that once you drop that Ref you just can't
//             access that instance
//           + probably also fine
//
// Even so: going to stick with this implementation for now.

/// A type that essentially wraps a [`Vec`] and ensures that instances of type
/// `T` that are placed within it live for `'store`.
///
/// This also ensures that there are no duplicate instances within the store.
///
/// Finally, this provides [`Ref`]s to the `T` instances stored within which
/// can then be turned into regular immutable or mutable references by
/// providing a reference to the object store instance the reference came from.
///
/// Because this type is append only (i.e. you cannot remove items; you can only
/// add them), the provided [`Ref`]s can outlive
//
// TODO: can we get a Ref and then use it with an ObjectStore that it did not
// originate from? or does the invariant nature of lifetimes on mutable
// references effectively prevent this?
///
/// ```compile_fail
/// # use aoc::object_store::ObjectStore;
/// let mut obj_store1 = Vec::new();
/// let mut obj_store2 = Vec::new();
///
/// let mut uno: ObjectStore<u8, ()> = ObjectStore::new(&mut obj_store1).unwrap();
/// let mut dos: ObjectStore<u8, ((),)> = ObjectStore::new(&mut obj_store2).unwrap();
///
/// let r1 = uno.insert(234).unwrap();
/// let r2 = dos.insert(123).unwrap();
///
/// // Shouldn't be able to use `r1` with `dos`:
/// assert_eq!(*dos.get(r1), 234);
/// ```
//
// TODO: does Ref need to store a "lifetime reference" corresponding to the
// parent object store so that it does not get dropped and then recreated while
// a Ref is out? For now to be cautious we do this. Actually we only reference
// the parent since then the other bound is implied. Actually only the Vec.
/// Lifetimes prevent this from happening. The reference can outlive the object
/// store but not the underlying Vec.
/// ```compile_fail
/// # use aoc::object_store::ObjectStore;
/// let mut v = Vec::new();
///
/// let mut a: ObjectStore<u8> = ObjectStore::new(&mut v).unwrap();
/// let r1 = a.insert(234).unwrap();
///
/// drop(a);
/// let mut b: ObjectStore<u8> = ObjectStore::new(&mut v).unwrap();
/// *b.get(r1);
/// ```
///
/// `UniqueMarker` is a way to make it so that you don't accidentally mix up
/// [`Ref`]s returned by different ObjectStore instances.
#[derive(Debug, PartialEq, Eq)]
pub struct ObjectStore<'store, T: Eq + Hash, UniqueMarker = ()> {
    // The `Vec` that the instances are actually stored in.
    store: &'store mut Vec<T>,

    // We want to have a `HashMap` somewhere so that we can quickly lookup new
    // entries to make sure we don't have duplicates.
    //
    // At the same time we don't have to have this `HashMap` actually _store_
    // instances of `T` since that'd be wasteful; we're already putting the
    // instances inside of `store`.
    //
    // But, we do want to be able to map from values of type `T` to references
    // to instances of `T`. What to do?
    //
    // `HashSet` (and `HashMap`) actually accept not just `T` but `Q: Borrow<T>`
    // when looking up something. We can use this to make it so that we can
    // look up a value of type `T` on a `HashSet` of `&T`s: when there are
    // duplicates this lookup will give us the existing pointer for `T` within
    // the store even though the `HashSet` does not store any instances of `T`.
    //
    // The reason this works is because the implementation of `PartialEq` for
    // references will dereference (i.e. "see through") the pointer effectively
    // meaning that the act of looking up a value in the `HashSet` will access
    // data in the store which is exactly what we wanted.
    //
    // Update: nevermind, we can't do the above because `store` is mutably
    // borrowed.
    //
    // So, we have to resort to using unsafe.
    map: HashSet<InnerRef<'store, T, UniqueMarker>>,
}

impl<'s, T: Eq + Hash, U> ObjectStore<'s, T, U> {
    /// Returns [`Err`] if the provided [`Vec`] is not empty.
    pub fn new(store: &'s mut Vec<T>) -> Result<Self, ()> {
        if !store.is_empty() {
            Err(())
        } else {
            Ok(Self {
                store,
                map: HashSet::new(),
            })
        }
    }

    pub fn insert(&mut self, obj: T) -> Result<Ref<'s, T, U>, (T, Ref<'s, T, U>)> {
        if let Some(r) = self.lookup(&obj) {
            Err((obj, r))
        } else {
            let idx = self.store.len();
            let r = Ref(idx, PhantomData);

            self.store.push(obj);
            let _added = self.map.insert(InnerRef(self.store as *const _, r));
            debug_assert!(_added);

            Ok(r)
        }
    }

    pub fn lookup(&self, obj: &T) -> Option<Ref<'s, T, U>> {
        self.map.get(obj).map(|t| t.1)
    }

    pub fn get(&self, r: Ref<'s, T, U>) -> &T {
        &self.store[r.0]
    }

    pub fn get_mut(&mut self, r: Ref<'s, T, U>) -> &mut T {
        &mut self.store[r.0]
    }
}

impl<'s, T: Eq + Hash, U> Index<Ref<'s, T, U>> for ObjectStore<'s, T, U> {
    type Output = T;

    fn index(&self, r: Ref<'s, T, U>) -> &T {
        self.get(r)
    }
}

impl<'s, T: Eq + Hash, U> IndexMut<Ref<'s, T, U>> for ObjectStore<'s, T, U> {
    fn index_mut(&mut self, r: Ref<'s, T, U>) -> &mut T {
        self.get_mut(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_duplicates() {
        let mut v = vec![];
        let mut s: ObjectStore<u8> = ObjectStore::new(&mut v).unwrap();

        let a = s.insert(234).unwrap();
        assert_eq!(s.insert(234), Err((234, a)));
    }

    #[test]
    fn linked_list() {
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        struct Node<'a> {
            prev: Option<Ref<'a, Node<'a>>>,
            next: Option<Ref<'a, Node<'a>>>,
            num: u8,
        }

        impl<'a> Node<'a> {
            fn new(num: u8) -> Self {
                Self {
                    prev: None,
                    next: None,
                    num,
                }
            }

            fn set_prev(&mut self, prev: Ref<'a, Node<'a>>) -> &mut Self {
                self.prev = Some(prev);
                self
            }

            fn set_next(&mut self, next: Ref<'a, Node<'a>>) -> &mut Self {
                self.next = Some(next);
                self
            }
        }

        let a = Node::new(0);
        let b = Node::new(1);
        let c = Node::new(2);

        let mut v = vec![];
        let mut s = ObjectStore::new(&mut v).unwrap();

        let a = s.insert(a).unwrap();
        let b = s.insert(b).unwrap();
        let c = s.insert(c).unwrap();

        // c → a → b
        // a → b → c
        // b → c → a
        s[a].set_prev(c).set_next(b);
        s[b].set_prev(a).set_next(c);
        s[c].set_prev(b).set_next(a);

        #[derive(Debug)]
        struct NodeIterator<'s, 'r>(&'r ObjectStore<'s, Node<'s>>, Ref<'s, Node<'s>>);

        impl<'s, 'r> Iterator for NodeIterator<'s, 'r> {
            type Item = &'r Node<'s>;

            fn next(&mut self) -> Option<&'r Node<'s>> {
                let curr = &self.0[self.1];

                if let Some(next) = curr.next {
                    self.1 = next;
                    Some(&self.0[next])
                } else {
                    None
                }
            }
        }

        assert_eq!(
            NodeIterator(&s, c).skip(3000).map(|n| n.num).next(),
            Some(0)
        );
    }

    #[test]
    fn non_empty_vec_is_an_error() {
        let mut f: Vec<u8> = vec![90];

        assert_eq!(ObjectStore::<u8>::new(&mut f), Err(()));
    }
}
