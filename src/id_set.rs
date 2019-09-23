use crate::key::Key;
use std::sync::Arc;

pub struct IdSetBuilder {
    data: Vec<(Key, Arc<[u32]>)>,
}

impl IdSetBuilder {
    pub(super) fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub(super) fn push(&mut self, key: Key, mut vec: Vec<u32>) -> Result<(), Vec<u32>> {
        if key
            > self
                .data
                .last()
                .map(|(key, _)| *key)
                .unwrap_or(Key::from_raw(0))
        {
            vec.push(u32::max_value());
            self.data.push((key, Arc::from(vec)));
            Ok(())
        } else {
            Err(vec)
        }
    }

    pub(super) fn build(self, gen: usize) -> IdSet {
        IdSet {
            gen,
            data: self.data,
        }
    }
}

pub struct IdSet {
    pub(super) gen: usize,
    pub(super) data: Vec<(Key, Arc<[u32]>)>,
}

impl IdSet {
    pub fn from_union(idset0: &IdSet, idset1: &IdSet) -> IdSet {
        if idset0.gen != idset1.gen {
            panic!("TODO: IdSet::gen doesn't match");
        }

        let mut data = Vec::<(Key, Arc<[u32]>)>::new();

        let mut it0 = idset0.data.iter();
        let mut it1 = idset1.data.iter();

        let mut n0 = it0.next();
        let mut n1 = it1.next();

        loop {
            match (n0, n1) {
                (Some((key0, set0)), Some((key1, _))) if key0 < key1 => {
                    data.push((*key0, set0.clone()));
                    n0 = it0.next();
                }
                (Some((key0, _)), Some((key1, set1))) if key0 > key1 => {
                    data.push((*key1, set1.clone()));
                    n1 = it1.next();
                }
                (Some((key0, set0)), Some((_key1, set1))) => {
                    let set = union_merge(set0, set1);
                    data.push((*key0, set));
                    n0 = it0.next();
                    n1 = it1.next();
                }
                (Some((key0, set0)), None) => {
                    data.push((*key0, set0.clone()));
                    n0 = it0.next();
                }
                (None, Some((key1, set1))) => {
                    data.push((*key1, set1.clone()));
                    n1 = it1.next();
                }
                (None, None) => {
                    break;
                }
            }
        }

        IdSet {
            gen: idset0.gen,
            data,
        }
    }

    pub fn from_intersection(idset0: &IdSet, idset1: &IdSet) -> IdSet {
        if idset0.gen != idset1.gen {
            panic!("TODO: IdSet::gen doesn't match");
        }

        let mut data = Vec::<(Key, Arc<[u32]>)>::new();

        let mut it0 = idset0.data.iter();
        let mut it1 = idset1.data.iter();

        let mut n0 = it0.next();
        let mut n1 = it1.next();

        loop {
            match (n0, n1) {
                (Some((key0, _)), Some((key1, _))) if key0 < key1 => {
                    n0 = it0.next();
                }
                (Some((key0, _)), Some((key1, _))) if key0 > key1 => {
                    n1 = it1.next();
                }
                (Some((key0, set0)), Some((_, set1))) => {
                    let set = intersection_merge(set0, set1);
                    if set.len() > 1 {
                        data.push((*key0, set));
                    }
                    n0 = it0.next();
                    n1 = it1.next();
                }
                (Some(_), None) => {
                    n0 = it0.next();
                }
                (None, Some(_)) => {
                    n1 = it1.next();
                }
                (None, None) => {
                    break;
                }
            }
        }

        IdSet {
            gen: idset0.gen,
            data,
        }
    }
}

fn union_merge(a0: &Arc<[u32]>, a1: &Arc<[u32]>) -> Arc<[u32]> {
    if Arc::ptr_eq(a0, a1) {
        return a0.clone();
    }

    unsafe {
        use std::alloc::{alloc, Layout};

        let data: *mut u32 = alloc(Layout::from_size_align_unchecked(
            4 * a0.len() + 4 * a1.len(),
            4,
        )) as _;
        *data = u32::max_value();

        let mut i0: *const u32 = (&*a0).as_ptr();
        let mut i1: *const u32 = (&*a1).as_ptr();
        let mut ic: *mut u32 = data;

        while *i0 != u32::max_value() || *i1 != u32::max_value() {
            if *i0 == *i1 {
                *ic = *i0;
                ic = ic.add(1);
                i0 = i0.add(1);
                i1 = i1.add(1);
            }

            if *i0 < *i1 {
                *ic = *i0;
                ic = ic.add(1);
                i0 = i0.add(1);
            }

            if *i1 < *i0 {
                *ic = *i1;
                ic = ic.add(1);
                i1 = i1.add(1);
            }
        }

        *ic = u32::max_value();
        ic = ic.add(1);

        let size = (ic as usize - data as usize) >> 2;
        let vec = Vec::from_raw_parts(data, size, a0.len() + a1.len());
        Arc::from(vec)
    }
}

fn intersection_merge(a0: &Arc<[u32]>, a1: &Arc<[u32]>) -> Arc<[u32]> {
    if Arc::ptr_eq(a0, a1) {
        return a0.clone();
    }

    unsafe {
        use std::alloc::{alloc, Layout};

        let data: *mut u32 = alloc(Layout::from_size_align_unchecked(
            4 * a0.len() + 4 * a1.len(),
            4,
        )) as _;
        *data = u32::max_value();

        let mut i0: *const u32 = (&*a0).as_ptr();
        let mut i1: *const u32 = (&*a1).as_ptr();
        let mut ic: *mut u32 = data;

        while *i0 != u32::max_value() && *i1 != u32::max_value() {
            if *i0 == *i1 {
                *ic = *i0;
                ic = ic.add(1);
                i0 = i0.add(1);
                i1 = i1.add(1);
            }

            if *i0 < *i1 {
                i0 = i0.add(1);
            }

            if *i1 < *i0 {
                i1 = i1.add(1);
            }
        }

        *ic = u32::max_value();
        ic = ic.add(1);

        let size = (ic as usize - data as usize) >> 2;
        let vec = Vec::from_raw_parts(data, size, a0.len() + a1.len());
        Arc::from(vec)
    }
}

#[test]
fn id_set_creation() {
    // create id set
    let id_set = {
        let mut id_set_builder = IdSetBuilder::new();
        id_set_builder
            .push(Key::from_raw(0b01), vec![1, 2, 3, 4, 5])
            .expect("Invalid order");
        id_set_builder
            .push(Key::from_raw(0b10), vec![3, 4, 5])
            .expect("Invalid order");
        id_set_builder
            .push(Key::from_raw(0b11), vec![5, 6, 7])
            .expect("Invalid order");
        id_set_builder.build(0)
    };

    // key 0b01
    let (ref key, ref set) = &id_set.data[0];
    assert!(*key == Key::from_raw(0b01));
    set.iter()
        .zip(
            vec![
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(u32::max_value()),
                None,
            ]
            .iter(),
        )
        .for_each(|(id1, id2)| assert!(*id1 == id2.unwrap()));

    // key 0b01
    let (ref key, ref set) = &id_set.data[1];
    assert!(*key == Key::from_raw(0b10));
    set.iter()
        .zip(vec![Some(3), Some(4), Some(5), Some(u32::max_value()), None].iter())
        .for_each(|(id1, id2)| assert!(*id1 == id2.unwrap()));

    // key 0b01
    let (ref key, ref set) = &id_set.data[2];
    assert!(*key == Key::from_raw(0b11));
    set.iter()
        .zip(vec![Some(5), Some(6), Some(7), Some(u32::max_value()), None].iter())
        .for_each(|(id1, id2)| assert!(*id1 == id2.unwrap()));
}

#[test]
fn id_set_union_merge() {
    // create 2 id sets
    let (id_set0, id_set1) = {
        let mut id_set_builder = IdSetBuilder::new();
        id_set_builder
            .push(Key::from_raw(0b001), vec![0, 2, 4, 6])
            .expect("Invalid order");
        id_set_builder
            .push(Key::from_raw(0b010), vec![1, 2, 3])
            .expect("Invalid order");
        id_set_builder
            .push(Key::from_raw(0b011), vec![1, 2, 3, 4, 5, 6])
            .expect("Invalid order");
        id_set_builder
            .push(Key::from_raw(0b100), vec![1, 2, 3])
            .expect("Invalid order");
        let id_set0 = id_set_builder.build(0);

        let mut id_set_builder = IdSetBuilder::new();
        id_set_builder
            .push(Key::from_raw(0b001), vec![1, 3, 5, 7])
            .expect("Invalid order");
        id_set_builder
            .push(Key::from_raw(0b010), vec![1, 2, 3])
            .expect("Invalid order");
        id_set_builder
            .push(Key::from_raw(0b011), vec![1, 2, 3])
            .expect("Invalid order");
        id_set_builder
            .push(Key::from_raw(0b101), vec![4, 5, 6])
            .expect("Invalid order");
        let id_set1 = id_set_builder.build(0);

        (id_set0, id_set1)
    };

    // union merge
    let merge = IdSet::from_union(&id_set0, &id_set1);

    // key 0b01
    let (ref key, ref set) = &merge.data[0];
    assert!(*key == Key::from_raw(0b01));
    set.iter()
        .zip(
            vec![
                Some(0),
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6),
                Some(7),
                Some(u32::max_value()),
                None,
            ]
            .iter(),
        )
        .for_each(|(id1, id2)| assert!(*id1 == id2.unwrap()));

    // key 0b10
    let (ref key, ref set) = &merge.data[1];
    assert!(*key == Key::from_raw(0b10));
    set.iter()
        .zip(vec![Some(1), Some(2), Some(3), Some(u32::max_value()), None].iter())
        .for_each(|(id1, id2)| assert!(*id1 == id2.unwrap()));

    // key 0b11
    let (ref key, ref set) = &merge.data[2];
    assert!(*key == Key::from_raw(0b11));
    set.iter()
        .zip(
            vec![
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6),
                Some(u32::max_value()),
                None,
            ]
            .iter(),
        )
        .for_each(|(id1, id2)| assert!(*id1 == id2.unwrap()));

    // key 0b100
    let (ref key, ref set) = &merge.data[3];
    assert!(*key == Key::from_raw(0b100));
    set.iter()
        .zip(vec![Some(1), Some(2), Some(3), Some(u32::max_value()), None].iter())
        .for_each(|(id1, id2)| assert!(*id1 == id2.unwrap()));

    // key 0b101
    let (ref key, ref set) = &merge.data[4];
    assert!(*key == Key::from_raw(0b101));
    set.iter()
        .zip(vec![Some(4), Some(5), Some(6), Some(u32::max_value()), None].iter())
        .for_each(|(id1, id2)| assert!(*id1 == id2.unwrap()));
}