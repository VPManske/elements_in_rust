pub mod elements {

    extern crate num;

    use std::cmp::{max};
    use std::ops::{ShrAssign, AddAssign, SubAssign};
    use self::num::{Zero, One, NumCast};

    //=============================================================================
    // 1.5 Regular Types

    // Regular types enable normal value semantics, except we require explicit
    // cloning rather than copying so the cost is visible in the code.
    pub trait Regular : PartialEq + Clone {}

    // A type is regular if it has assignment, equality, and is clonable.
    impl<I> Regular for I where I : PartialEq + Clone {}

    //=============================================================================
    // 2.1 Integers

    // This is an incomplete implementation of the Integer concept with just the 
    // functions necessary for the iterator algorithms below.
    pub trait Integer : num::Integer
    where Self : Regular + NumCast + ShrAssign<Self> + AddAssign<Self> + SubAssign<Self>
    {
        fn two() -> Self where Self : NumCast {
            Self::from(2).unwrap()
        }
        fn is_two(&self) -> bool where Self : NumCast + PartialEq {
            *self == Self::from(2).unwrap()
        }
        fn successor_assign(&mut self) {
            *self += Self::one()
        }
        fn predecessor_assign(&mut self) {
            *self -= Self::one();
        }
        fn half_nonnegative_assign(&mut self) {
            *self >>= Self::one()
        }
        fn successor(mut self) -> Self {
            self.successor_assign();
            self
        }
        fn predecessor(mut self) -> Self { // where Self : Sized {
            self.predecessor_assign();
            self
        }
        fn half_nonnegative(mut self) -> Self {
            self.half_nonnegative_assign();
            self
        }
    }

    impl<I> Integer for I
    where I : num::Integer + NumCast + Regular
        + ShrAssign<I> + AddAssign<I> + SubAssign<I> {}

    //=============================================================================
    // 6.1 Readability

    // Readability is roughly equivalent to a read-only reference in Rust, but also
    // represents read-only access to containers. Ideally this is a read-only
    // version of Rusts 'Deref' trait, but we would like a value to derefence to
    // itself.

    pub trait Reference {
        type ValueType : Regular;
    }

    pub trait Readable : Reference {
        fn source(&self) -> &Self::ValueType;
    }

    pub trait Writable : Reference { // we cannot enforce a write only refernce in Rust
        fn sink(&self) -> &mut Self::ValueType;
    }

    pub trait Mutable : Reference + Readable + Writable {
        fn deref(&self) -> &mut Self::ValueType;
    }

    //-----------------------------------------------------------------------------
    // 6.2 Iterators

    // A plain Iterator is one which can only pass through the data once. To
    // enforce this it must not be copyable, and successor needs to consume the
    // current iterator to return a new one. This means that we define increment
    // which mutates the iterator instead of successor, as we can then implement
    // successor without copying the iterator or moving it out of the borrowed
    // context.
    pub trait Iterator : PartialEq {
        type DistanceType : Integer;
        fn successor_assign(&mut self);

        fn successor(mut self) -> Self where Self : Sized {
            self.successor_assign();
            self
        }

        // 6.3 Ranges

        fn add_assign(&mut self, mut n : Self::DistanceType) {
            // Precondition: n >= 0 && weak_range(f, n)
            while n != Self::DistanceType::zero() {
                n.predecessor_assign();
                self.successor_assign();
            }
        }

        fn add(mut self, n : Self::DistanceType) -> Self where Self : Sized {
            self.add_assign(n);
            self
        }

        fn dif(&self, mut f : Self) -> Self::DistanceType where Self : Sized {
            // Precondition: bounded_range(f, l)
            let mut n = Self::DistanceType::zero();
            while f != *self {
                n = n.successor();
                f = f.successor();
            }
            n
        }
    }       

    //-----------------------------------------------------------------------------
    // 6.4 Readable Ranges

    pub fn for_each<I, P>(mut f : I, l : &I, mut p : P) -> P
    where I : Iterator + Readable, P : FnMut(&I::ValueType) {
        // Precondition: readable_bounded_range(f, l)
        while f != *l {
            p(f.source());
            f = f.successor();
        }
        p
    }

    pub fn find<I>(mut f : I, l : &I, x : I::ValueType) -> I
    where I : Iterator + Readable {
        // Precondition: readable_bounded_range(f, l)
        while (f != *l) && (*(f.source()) != x) {
            f = f.successor();
        }
        f 
    }
    
    pub fn find_if<I, P>(mut f : I, l : &I, mut p : P) -> I
    where I : Iterator + Readable, P : FnMut(&I::ValueType) -> bool {
        // Precondition: readable_bounded_range(f, l)
        while (f != *l) && !p(f.source()) {
            f = f.successor();
        }
        f 
    }

    pub fn find_if_not<I, P>(mut f : I, l : &I, mut p : P) -> I
    where I : Iterator + Readable, P : FnMut(&I::ValueType) -> bool {
        // Precondition: readable_bounded_range(f, l)
        while (f != *l) && p(f.source()) {
            f = f.successor();
        }
        f
    }

    pub fn count_if<I, J, P>(mut f : I, l : &I, mut p : P, mut j : J) -> J
    where I : Iterator + Readable, J : Integer, P : FnMut(&I::ValueType) -> bool {
        // Precondition: readable_bounded_range(f, l)
        while f != *l {
            if p(f.source()) {
                j = j.successor();
            }
            f = f.successor();
        }
        j
    }

    pub fn count_if_from_zero<I, P>(f : I, l : &I, p : P) -> I::DistanceType
    where I : Iterator + Readable, P : FnMut(&I::ValueType) -> bool {
        // Precondition: readable_bounded_range(f, l)
        count_if(f, l, p, I::DistanceType::zero())
    }

    pub fn fold<I, Op>(mut f : I, l : &I, mut op : Op, mut r : I::ValueType) -> I::ValueType
    where I : Iterator + Readable, Op : FnMut(&I::ValueType, &I::ValueType) -> I::ValueType {
        // Precondition: readable_bounded_range(f, l)
        // Precondition: partially_associative(op) 
        while f != *l {
            r = op(&r, f.source());
            f = f.successor();
        }
        r
    }

    pub fn reduce_nonempty<I, Op, F, D>(mut f : I, l : &I, mut op : Op, mut fun : F) -> D 
    where I : Iterator + Readable, Op : FnMut(D, D) -> D, F : FnMut(&I) -> D {
        // Precondition: readable_bounded_range(f, l)
        // Precondition: partially_associative(op)
        // Precondition: forall x in [f,l) fun(x) is defined 
        let mut r = fun(&f);
        f = f.successor();
        while f != *l {
            r = op(r, fun(&f));
            f = f.successor();
        }
        r
    }

    pub fn reduce<I, Op, F, D>(f : I, l : &I, op : Op, fun : F, z : D) -> D
    where D : Regular, I : Iterator + Readable, Op : FnMut(D, D) -> D, F : FnMut(&I) -> D {
        // Precondition: readable_bounded_range(f, l)
        // Precondition: partially_associative(op)
        // Precondition: forall x in [f,l) fun(x) is defined 
        if f == *l {
            z
        } else {
            reduce_nonempty(f, l, op, fun)
        }
    }

    pub fn reduce_nonzeroes<I, Op, F, D>(mut f : I, l : &I, mut op : Op, mut fun : F, z : D) -> D 
    where D : Regular, I : Iterator + Readable, Op : FnMut(D, D) -> D, F : FnMut(&I) -> D {
        // Precondition: readable_bounded_range(f, l)
        // Precondition: partially_associative(op)
        // Precondition: forall x in [f,l) fun(x) is defined 
        let mut x : D;
        while {
            if f == *l {
               return z;
            }
            x = fun(&f);
            f = f.successor();
            x == z
        } {}
        while f != *l {
            let y = fun(&f);
            if y != z {
                x = op(x, y);
            }
            f = f.successor();
        }
        x
    }

    pub fn for_each_n<I, P>(mut f : I, mut n : I::DistanceType, mut p : P) -> (P, I)
    where I : Iterator + Readable, P : FnMut(&I::ValueType) {
        // Precondition: readable_weak_range(f, n)
        while I::DistanceType::zero() != n {
            n = n.predecessor();
            p(f.source());
            f = f.successor();
        }
        (p, f)
    } 

    pub fn find_n<I>(mut f : I, mut n : I::DistanceType, x : I::ValueType) -> (I, I::DistanceType)
    where I : Iterator + Readable {
        // Precondition: weak_range(f, n)
        while n != I::DistanceType::zero() && *(f.source()) != x {
            n = n.predecessor();
            f = f.successor();
        }
        (f, n)
    }

    pub fn find_if_unguarded<I, P>(mut f : I, mut p : P) -> I
    where I : Iterator + Readable, P : FnMut(&I::ValueType) -> bool {
        // Precondition: exists l . readable_bounded_range(f, l) && some(f, j, p)
        while !p(f.source()) {
            f = f.successor();
        }
        f
        // Postcondition: p(f.source())
    }

    pub fn find_mismatch<I0, I1, R, V>(mut f0 : I0, l0 : &I0, mut f1 : I1, l1 : &I1, mut r : R) -> (I0, I1)
    where I0 : Iterator + Readable<ValueType = V>, I1 : Iterator + Readable<ValueType = V>,
    R : FnMut(&V, &V) -> bool, V : Regular {
        // Precondition: readable_bounded_range(f0, l0)
        // Precondition: readable_bounded_range(f1, l1)
        while f0 != *l0 && f1 != *l1 && r(f0.source(), f1.source()) {
            f0 = f0.successor();
            f1 = f1.successor();
        }
        (f0, f1)
    }

    // Note: this algorithm needs to clone the data as the iterator is a single pass 
    // iterator. That means you cannot hold a reference to the location pointed to by
    // 'f' once 'f' has been incremented. To do so would effectively allow the iterator
    // to be copied, which would break the invarient conditions. 
    pub fn find_adjacent_mismatch<I, R>(mut f : I, l : &I, mut r : R) -> I
    where I : Iterator + Readable, R : FnMut(&I::ValueType, &I::ValueType) -> bool {
        // Precondition: readable_bounded_range(f, l)
        if f != *l {
            //let mut x : I::ValueType = (*f.source()).clone();
            let mut x = (*f.source()).clone();
            f = f.successor();
            while (f != *l) && r(&x, f.source()) {
                x = (*f.source()).clone();
                f = f.successor();
            }
        }
        f
    }

    //-----------------------------------------------------------------------------
    // 6.5 Increasing Ranges

    pub fn relation_preserving<I, R>(f : I, l : &I, r : R) -> bool
    where I : Iterator + Readable, R : FnMut(&I::ValueType, &I::ValueType) -> bool {
        // Precondition: readable_bounded_range(f, l)
        *l == find_adjacent_mismatch(f, l, r)
    }

    pub fn strictly_increasing_range<I, R>(f : I, l : &I, r : R) -> bool
    where I : Iterator + Readable, R : FnMut(&I::ValueType, &I::ValueType) -> bool {
        // Precondition: readable_bounded_range(f, l) && weak_ordering(r)
        relation_preserving(f, l, r)
    }

    pub fn complement_of_converse<'a, R, A>(mut r : R) -> Box<FnMut(&A, &A) -> bool + 'a>
    where R : FnMut(&A, &A) -> bool + 'a {
        Box::new(move |a, b| !r(b, a))
    }

    pub fn increasing_range<I, R>(f : I, l : &I, r : R) -> bool
    where I : Iterator + Readable, R : FnMut(&I::ValueType, &I::ValueType) -> bool + Sized {
        // Precondition: readable_bounded_range(f, l) && weak_ordering(r)
        relation_preserving(f, l, &mut *complement_of_converse(r))
    }

    pub fn partitioned<I, P>(f : I, l : &I, mut p : P) -> bool
    where I : Iterator + Readable, P : FnMut(&I::ValueType) -> bool {
        // Precondition: readable_bounded_range(f, l)
        let g = find_if(f, l, &mut p);
        *l == find_if_not(g, l, p)
    }

    //-----------------------------------------------------------------------------
    // 6.6 Forward Iterators

    // A Forward Iterator is an Iterator that is also a Regular type, this means it
    // must behave like a value, and be copyable, assignable (storable) in addition
    // to having an equality operator.
    pub trait ForwardIterator : Iterator + Regular {}

    // Because a forward iterator is copyable we cam clone the iterator instead
    // of having to clone the data.
    pub fn find_adjacent_mismatch_forward<I, R>(mut f : I, l : &I, mut r : R) -> I
    where I : ForwardIterator + Readable, R : FnMut(&I::ValueType, &I::ValueType) -> bool {
        // Precondition: readable_bounded_range(f, l)
        if f != *l {
            let mut t = f.clone();
            f = f.successor();
            while f != *l && r(t.source(), f.source()) {
                t = t.successor();
                f = f.successor();
            }
        }
        f
    }

    //pub fn partition_point_n<I, P>(mut f : I, mut n : I::DistanceType, mut p : P) -> I
    pub fn partition_point_n<I, P>(mut f : I, mut n : I::DistanceType, mut p : P) -> I
    where I : ForwardIterator + Readable, P : FnMut(&I::ValueType) -> bool {
        // Precondition: readable_counted_range(f, n) && partitioned_n(f, n, p)
        while !n.is_zero() {
            let h : I::DistanceType = n.clone().half_nonnegative();
            let m : I = f.clone().add(h.clone());
            if p(m.source()) {
                n = h;
            } else {
                n = n - h.successor();
                f = m.successor();
            }
        }
        f
    }

    pub fn partition_point<I, P>(f : I, l : I, p : P) -> I
    where I : ForwardIterator + Readable, P : FnMut(&I::ValueType) -> bool {
        // Precondition: readable_bounded_range(f, n) && partitioned(f, l, p)
        partition_point_n(f.clone(), l.dif(f), p)
    }

    pub fn lower_bound_predicate<'a, R, D>(a : &'a D, mut r : R) -> Box<FnMut(&D) -> bool + 'a>
    where R : FnMut(&D, &D) -> bool + 'a {
        Box::new(move |x| !r(x, a))
    }

    pub fn lower_bound_n<I, R>(f : I, n : I::DistanceType, a : &I::ValueType, r : R) -> I
    where I : Readable + ForwardIterator, R : FnMut(&I::ValueType, &I::ValueType) -> bool {
        // Precondition: weak_ordering(r) && increasing_counted_range(f, n, r)
        partition_point_n(f, n, &mut *lower_bound_predicate(a, r))
    }

    pub fn upper_bound_predicate<'a, R, D>(a : &'a D, mut r : R) -> Box<FnMut(&D) -> bool + 'a>
    where R : FnMut(&D, &D) -> bool + 'a {
        Box::new(move |x| r(a, x))
    }

    pub fn upper_bound_n<I, R>(f : I, n : I::DistanceType, a : &I::ValueType, r : R) -> I
    where I : Readable + ForwardIterator, R : FnMut(&I::ValueType, &I::ValueType) -> bool {
        // Precondition: weak_ordering(r) && increasing_counted_range(f, n, r)
        partition_point_n(f, n, &mut *upper_bound_predicate(a, r))
    }

    //-----------------------------------------------------------------------------
    // 6.7 Indexed Iterators

    pub trait IndexedIterator : ForwardIterator {}

    //-----------------------------------------------------------------------------
    // 6.8 Bidirectional Iterators

    pub trait BidirectionalIterator : ForwardIterator {
        fn predecessor_assign(&mut self);

        fn predecessor(mut self) -> Self {
            self.predecessor_assign();
            self
        }

        fn sub_assign(&mut self, mut n : Self::DistanceType) {
            // Precondition: n >= 0 && exists f . f in I => weak_range(f, n) && l = f + n
            while !n.is_zero() {
                n.predecessor_assign();
                self.predecessor_assign();
            }
        }


        fn sub(mut self, n : Self::DistanceType) -> Self {
            self.sub_assign(n);
            self
        }
    }

    pub fn find_backward_if<I, P>(f : &I, mut l : I, mut p : P) -> I
    where I : Readable + BidirectionalIterator, P : FnMut(&I::ValueType) -> bool {
        if *f != l {
            l = l.predecessor();
            while *f != l && !p(l.source()) {
                l = l.predecessor();
            }
        }
        l
    }

    //-----------------------------------------------------------------------------
    // 6.9 Random Access Iterators
    
    pub trait RandomAccessIterator : IndexedIterator + BidirectionalIterator {
        type DifferenceType : Integer;
        fn less_than(&self, y: &Self) -> bool;
    }

    //=============================================================================
    // 7.1 Bifurcate Coordinates
   
    pub trait BifurcateCoordinate : Regular {
        type WeightType : Integer;
        fn empty(&self) -> bool;
        fn has_left_successor(&self) -> bool;
        fn has_right_successor(&self) -> bool;
        fn left_successor_assign(&mut self);
        fn right_successor_assign(&mut self);

        fn left_successor(mut self) -> Self {
            self.left_successor_assign();
            self
        }

        fn right_successor(mut self) -> Self {
            self.right_successor_assign();
            self
        }
    }

    pub fn weight_recursive<C>(c : C) -> C::WeightType where C : BifurcateCoordinate {
        // Precondition: tree(c)
        if c.empty() {return C::WeightType::zero();}
        let mut l = C::WeightType::zero();
        let mut r = C::WeightType::zero();
        if c.has_left_successor() {
            l = weight_recursive(c.clone().left_successor());
        }
        if c.has_right_successor() {
            r = weight_recursive(c.right_successor());
        }
        (l + r).successor()
    }

    pub fn height_recursive<C>(c : C) -> C::WeightType where C : BifurcateCoordinate {
        // Precondition: tree(c)
        if c.empty() {return C::WeightType::zero();}
        let mut l = C::WeightType::zero();
        let mut r = C::WeightType::zero();
        if c.has_left_successor() {
            l = height_recursive(c.clone().left_successor());
        }
        if c.has_right_successor() {
            r = height_recursive(c.right_successor());
        }
        max(l, r).successor()
    }

    #[derive(PartialEq, PartialOrd)]
    pub enum Visit {Pre, In, Post}

    pub fn traverse_nonempty<C, P>(c : C, mut p : P) -> P
    where C : BifurcateCoordinate, P : FnMut(Visit, &C) {
        // Precondition: tree(c) /\ ~empty(c)
        p(Visit::Pre, &c);
        if c.has_left_successor() {
            p = traverse_nonempty(c.clone().left_successor(), p);
        }
        p(Visit::In, &c);
        if c.has_right_successor() {
            p = traverse_nonempty(c.clone().right_successor(), p);
        }
        p(Visit::Post, &c);
        return p;
    }

    //-----------------------------------------------------------------------------
    // 7.2 Bidirectional Bifurcate Coordinates

    pub trait BidirectionalBifurcateCoordinate : BifurcateCoordinate {
        fn has_predecessor(&self) -> bool;
        fn predecessor_assign(&mut self);
        fn predecessor(mut self) -> Self {
            self.predecessor_assign();
            self
        }
    }

    pub fn is_left_successor<T>(j : &T) -> bool
    where T : BidirectionalBifurcateCoordinate {
        // Precondition: has_predecessor(j)
        let i = j.clone().predecessor();
        i.has_left_successor() && i.left_successor() == *j
    }

    pub fn is_right_successor<T>(j : &T) -> bool
    where T : BidirectionalBifurcateCoordinate {
        // Precondition: has_predecessor(j)
        let i = j.clone().predecessor();
        i.has_right_successor() && i.right_successor() == *j
    }

    pub fn traverse_step<C>(v : &mut Visit, c : &mut C) -> i32
    where C : BidirectionalBifurcateCoordinate {
        // Precondition: has_predecessor(v) || v != post
        match *v {
            Visit::Pre => {
                if c.has_left_successor() {
                    c.left_successor_assign();
                    return 1;
                }
                *v = Visit::In;
                return 0;   
            },
            Visit::In => {
                if c.has_right_successor() {
                    *v = Visit::Pre;
                    c.right_successor_assign();
                    return 1;
                }
                *v = Visit::Post;
                return 0;
            },
            Visit::Post => {
                if is_left_successor(c) {
                    *v = Visit::In;
                }
                c.predecessor_assign();
                return -1;
            }
        }
    }

    pub fn reachable<C>(mut x : C, y : C) -> bool
    where C : BidirectionalBifurcateCoordinate {
        // Precondition: tree(c)
        if x.empty() {
            return false;
        } 
        let root = x.clone();
        let mut v = Visit::Pre;
        loop {
            if x == y {
                return true;
            }
            traverse_step(&mut v, &mut x);
            if x != root && v != Visit::Post {
                return false;
            }
        }
    }

    pub fn weight<C>(mut c : C) -> C::WeightType 
    where C : BidirectionalBifurcateCoordinate {
        // Precondition: tree(c)
        if c.empty() {
            return C::WeightType::zero();
        }
        let root = c.clone();
        let mut v = Visit::Pre;
        let mut n = C::WeightType::one();
        loop {
            traverse_step(&mut v, &mut c);
            if v == Visit::Pre {
                n = n.successor();
            }
            if c == root && v == Visit::Post {
                return n;
            }
        }
    }

    pub fn height<C>(mut c : C) -> C::WeightType
    where C : BidirectionalBifurcateCoordinate {
        // Precondition: tree(c)
        if c.empty() {
            return C::WeightType::zero();
        }
        let root = c.clone();
        let mut v = Visit::Pre;
        let mut m = C::WeightType::one(); // Invariant: m is height of current pre-visit.
        let mut n = C::WeightType::one(); // Invarient: n is max of height of pre=visits so far.
        loop {
            m = (m - C::WeightType::one())
                + num::NumCast::from(traverse_step(&mut v, &mut c)).unwrap()
                + C::WeightType::one();
            n = max(&n, &m).clone();
            if c == root && v == Visit::Post {
                return n;
            }
        }
    }

    pub fn traverse<C, P>(mut c : C, mut p : P) -> P
    where C : BidirectionalBifurcateCoordinate, P : FnMut(&Visit, &C) {
        // Precondition: tree(c)
        if c.empty() {
            return p;
        }
        let root = c.clone();
        let mut v = Visit::Pre;
        p(&v, &c);
        loop {
            traverse_step(&mut v, &mut c);
            p(&v, &c);
            if c == root && v == Visit::Post {
                return p;
            }
        }
    }

    //-----------------------------------------------------------------------------
    // 7.4 Isomorphism, Equivalence and Ordering

    pub fn bifurcate_isomorphic_nonempty<C0, C1>(c0 : C0, c1 : C1) -> bool
    where C0 : BifurcateCoordinate, C1 : BifurcateCoordinate {
        // Precondition: tree(c0) && tree(c1) && !empty(c0) && !empty(c1)
        if c0.has_left_successor() {
            if c1.has_left_successor() {
                if !bifurcate_isomorphic_nonempty(c0.clone().left_successor(), c1.clone().left_successor()) {
                    return false;
                }
            } else {
                return false;
            }
        } else if c1.has_left_successor() {
            return false;
        }
        if c0.has_right_successor() {
            if c1.has_right_successor() {
                if !bifurcate_isomorphic_nonempty(c0.right_successor(), c1.right_successor()) {
                    return false;
                } 
            } else {
                return false;
            }
        } else if c1.has_right_successor() {
            return false;
        }
        return true;
    }

    pub fn bifurcate_isomorphic<C0, C1>(mut c0 : C0, mut c1 : C1) -> bool
    where C0 : BidirectionalBifurcateCoordinate, C1 : BidirectionalBifurcateCoordinate {
        // Precondition: tree(c0) && tree(c1)
        if c0.empty() {
            return c1.empty();
        }
        if c1.empty() {
            return false;
        }
        let root0 = c0.clone();
        let mut v0 = Visit::Pre;
        let mut v1 = Visit::Pre;
        loop {
            traverse_step(&mut v0, &mut c0);
            traverse_step(&mut v1, &mut c1);
            if v0 != v1 {return false;}
            if c0 == root0 && v0 == Visit::Post {return true;}
        }
    }

    pub fn lexicographical_equivalent<I0, I1, R, V>(f0 : I0, l0 : &I0, f1 : I1, l1 : &I1, r : R) -> bool
    where I0 : Readable<ValueType = V> + Iterator, I1 : Readable<ValueType = V> + Iterator,
    R : FnMut(&V, &V) -> bool, V : Regular {
        // Precondition: readable_bounded_range(f0, l0)
        // Precondition: readable_bounded_range(f1, l1)
        // Precondition: equivalence(r)
        let p = find_mismatch(f0, l0, f1, l1, r);
        p.0 == *l0 && p.1 == *l1
    }

    pub fn equal<T>(x : &T, y : &T) -> bool where T : Regular {
        x == y
    }
        
    pub fn lexicographical_equal<I0, I1, V>(f0 : I0, l0 : &I0, f1 : I1, l1 : &I1) -> bool
    where I0 : Readable<ValueType = V> + Iterator, I1 : Readable<ValueType = V> + Iterator, V : Regular {
        lexicographical_equivalent(f0, l0, f1, l1, equal::<V>)
    }

    pub fn bifurcate_equivalent_nonempty<C0, C1, R>(c0 : C0, c1 : C1, r : &mut R) -> bool
    where C0 : Readable + BifurcateCoordinate, C1 : Readable + BifurcateCoordinate,
    R : FnMut(&C0::ValueType, &C1::ValueType) -> bool {
        // Precondition: readable_tree(c0) && readable_tree(c1)
        // Precondition: !empty(c0) && !empty(c1)
        // Precondition: equivalence(r)
        if !r(c0.source(), c1.source()) {
            return false;
        }
        if c0.has_left_successor() {
            if c1.has_left_successor() {
                if !bifurcate_equivalent_nonempty(c0.clone().left_successor(), c1.clone().right_successor(), r) {
                    return false;
                }
            } else {
                return false;
            }
        } else if c1.has_left_successor() {
            return false;
        }
        if c0.has_right_successor() {
            if c1.has_right_successor() {
                if !bifurcate_equivalent_nonempty(c0.left_successor(), c1.right_successor(), r) {
                    return false;
                }
            } else {
                return false;
            }
        } else if c1.has_right_successor() {
            return false;
        }
        return true;
    }

    pub fn bifurcate_equivalent<C0, C1, R>(mut c0 : C0, mut c1 : C1, mut r : R) -> bool
    where C0 : Readable + BidirectionalBifurcateCoordinate, C1 : Readable + BidirectionalBifurcateCoordinate,
    R : FnMut(&C0::ValueType, &C1::ValueType) -> bool {
        // Precondition: readable_tree(c0) && readable_tree(c1)
        // Precondition: equivalence(r)
        if c0.empty() {
            return c1.empty()
        }
        if c1.empty() {
            return false;
        }
        let root0 = c0.clone();
        let mut v0 = Visit::Pre;
        let mut v1 = Visit::Pre;
        loop {
            if v0 == Visit::Pre && !r(c0.source(), c1.source()) {
                return false;
            }
            traverse_step(&mut v0, &mut c0);
            traverse_step(&mut v1, &mut c1);
            if v0 != v1 {
                return false;
            }
            if c0 == root0 && v0 == Visit::Post {
                return true;
            }
        }
    }

    pub fn lexicographical_compare<I0, I1, R>(mut f0 : I0, l0 : &I0, mut f1 : I1, l1 : &I1, mut r : R) -> bool
    where I0 : Readable + Iterator, I1 : Readable<ValueType = I0::ValueType> + Iterator,
    R : FnMut(&I0::ValueType, &I0::ValueType) -> bool {
        // Precondition: readable_bounded_range(f0, l0)
        // Precondition: readable_bounded_range(f1, l1)
        // Precondition: weak_ordering(r)
        loop {
            if f1 == *l1 {
                return false;
            }
            if f0 == *l0 {
                return true;
            }
            if r(f0.source(), f1.source()) {
                return true;
            }
            if r(f1.source(), f0.source()) {
                return false;
            }
            f0.successor_assign();
            f1.successor_assign();
        }
    }

    pub fn less<T>(x : &T, y : &T) -> bool where T : PartialOrd {
        x < y
    }

    pub fn lexicographical_less<I0, I1>(f0 : I0, l0 : &I0, f1 : I1, l1 : &I1) -> bool
    where I0 : Readable + Iterator, I1 : Readable<ValueType = I0::ValueType> + Iterator,
    I0::ValueType : PartialOrd {
        lexicographical_compare(f0, l0, f1, l1, less::<I0::ValueType>)
    }

    pub fn bifurcate_compare<C0, C1, R>(mut c0 : C0, mut c1 : C1, mut r : R) -> bool
    where C0 : Readable + BidirectionalBifurcateCoordinate,
    C1 : Readable<ValueType = C0::ValueType> + BidirectionalBifurcateCoordinate,
    R : FnMut(&C0::ValueType, &C1::ValueType) -> bool {
        // Precondition: readable_tree(c0) && readable_tree(c1)
        // Precondition: weak_ordering(r)
        if c1.empty() {
            return false;
        }
        if c0.empty() {
            return true;
        }
        let root0 = c0.clone();
        let mut v0 = Visit::Pre;
        let mut v1 = Visit::Pre;
        loop {
            if v0 == Visit::Pre {
                if r(c0.source(), c1.source()) {
                    return true;
                }
                if r(c1.source(), c0.source()) {
                    return false;
                }
            }
            traverse_step(&mut v0, &mut c0);
            traverse_step(&mut v1, &mut c1);
            if v0 != v1 {
                return v0 > v1;
            }
            if c0 == root0 && v0 == Visit::Post {
                return false;
            }
        }
    }

    //=============================================================================
    // 8.1 Linked Iterators

    pub trait ForwardLinker {
        type IteratorType : ForwardIterator;
        extern "rust-call" fn call_mut(&mut self, Self::IteratorType, Self::IteratorType);
    }

    
   
    //-----------------------------------------------------------------------------
    // 9.4 Swapping Ranges

    pub fn exchange_values<I0, I1>(x : &I0, y : &I1)
    where I0 : Mutable<ValueType = I1::ValueType>, I1 : Mutable {
        let t : I0::ValueType = x.source().clone();
        *x.sink() = y.source().clone();
        *y.sink() = t;
    }

    //-----------------------------------------------------------------------------
    // 10.3 Reverse Algorithms

    pub fn reverse_bidirectional<I>(mut f : I, mut l : I)
    where I : Mutable + BidirectionalIterator {
        loop {
            if f == l {return;}
            l = l.predecessor();
            if f == l {return;}
            exchange_values(&f, &l);
            f = f.successor();
        }
    }
}

//=============================================================================

#[cfg(test)]
mod test {

    extern crate num;

    use std::fmt::*;
    use std::ops::*;
    use std::marker::PhantomData;
    use elements::*;
    use std::mem;

    //-----------------------------------------------------------------------------
    // Define Immutable Slice Iterator

    trait Iterable<'a> {
        type IteratorType : Iterator;
        fn begin(&'a self) -> Self::IteratorType;
        fn end(&'a self) -> Self::IteratorType;
    }


    #[derive(Clone, PartialEq, Debug)]
    struct SliceIterator<'a, T> where T : 'a {
        ptr : *const T,
        phantom : PhantomData<&'a ()>
    } 

    impl<'a, T> Iterable<'a> for [T] where T : PartialEq + Clone + Debug + 'a {
        type IteratorType = SliceIterator<'a, T>;
        fn begin(&'a self) -> Self::IteratorType {
            SliceIterator::new(self.first().unwrap())
        }
        fn end(&'a self) -> Self::IteratorType {
            SliceIterator::new(self.last().unwrap()).successor()
        }
    }

    impl<'a, T> SliceIterator<'a, T> {
        fn new(r : &'a T) -> SliceIterator<'a, T> {
            SliceIterator {
                ptr : r,
                phantom : PhantomData
            }
        }
    }

    impl<'a, T> Display for SliceIterator<'a, T> where T : Display {
        fn fmt(&self, f : &mut Formatter) -> Result {
            write!(f, "{}", (self.ptr as *const T) as usize)
        }
    }

    impl<'a, T> Reference for SliceIterator<'a, T> where T : Regular {
        type ValueType = T;
    }

    impl<'a, T> Readable for SliceIterator<'a, T> where T : Regular {
        fn source(&self) -> &T {
            unsafe {&*self.ptr}
        }
    }

    impl<'a, T> Iterator for SliceIterator<'a, T> where SliceIterator<'a, T> : PartialEq, T : Regular {
        type DistanceType = usize;
        fn successor_assign(&mut self) {
            unsafe {self.ptr = self.ptr.offset(1)};
        }
        fn add_assign(&mut self, n : Self::DistanceType) {
            let m : isize = num::NumCast::from(n).unwrap();
            unsafe {self.ptr = self.ptr.offset(m)};
        }
        fn dif(&self, f : Self) -> <Self as Iterator>::DistanceType {
            num::NumCast::from(
                (self.ptr as usize - f.ptr as usize) / mem::size_of::<T>()
            ).unwrap()
        }
    }

    // This iterator is copyable.
    impl<'a, T> ForwardIterator for SliceIterator<'a, T> where SliceIterator<'a, T> : Iterator + Regular {}

    // This iterator provides efficient add and sub operators.
    impl<'a, T> IndexedIterator for SliceIterator<'a, T> where SliceIterator<'a, T> : ForwardIterator {}

    // This iterator is bidirectional.
    impl<'a, T> BidirectionalIterator for SliceIterator<'a, T> where SliceIterator<'a, T> : ForwardIterator {
        fn predecessor_assign(&mut self) {
            unsafe {self.ptr = self.ptr.offset(-1)}
        }
        fn sub_assign(&mut self, n : Self::DistanceType) {
            let m : isize = num::NumCast::from(n).unwrap();
            unsafe {self.ptr = self.ptr.offset(-m)}
        }
    }

    impl<'a, T> RandomAccessIterator for SliceIterator<'a, T>
    where SliceIterator<'a, T> : IndexedIterator + BidirectionalIterator {
        type DifferenceType = isize;
        fn less_than(&self, y : &Self) -> bool {
            (self.ptr as usize) < (y.ptr as usize)
        }
    }

    //-----------------------------------------------------------------------------
    // Define Mutable Slice Iterator

    trait IterableMut<'a> {
        type IteratorTypeMut : Iterator;
        fn begin_mut(&'a mut self) -> Self::IteratorTypeMut;
        fn end_mut(&'a mut self) -> Self::IteratorTypeMut;
    }


    #[derive(Clone, PartialEq, Debug)]
    struct SliceIteratorMut<'a, T> where T : 'a {
        ptr : *mut T,
        phantom : PhantomData<&'a ()>
    } 

    impl<'a, T> IterableMut<'a> for [T] where T : PartialEq + Clone + Debug + 'a {
        type IteratorTypeMut = SliceIteratorMut<'a, T>;
        fn begin_mut(&'a mut self) -> Self::IteratorTypeMut {
            SliceIteratorMut::new(self.first_mut().unwrap())
        }
        fn end_mut(&'a mut self) -> Self::IteratorTypeMut {
            SliceIteratorMut::new(self.last_mut().unwrap()).successor()
        }
    }

    impl<'a, T> SliceIteratorMut<'a, T> {
        fn new(r : &'a mut T) -> SliceIteratorMut<'a, T> {
            SliceIteratorMut {
                ptr : r,
                phantom : PhantomData
            }
        }
    }

    impl<'a, T> Display for SliceIteratorMut<'a, T> where T : Display {
        fn fmt(&self, f : &mut Formatter) -> Result {
            write!(f, "{}", (self.ptr as *mut T) as usize)
        }
    }

    impl<'a, T> Reference for SliceIteratorMut<'a, T> where T : Regular {
        type ValueType = T;
    }

    impl<'a, T> Readable for SliceIteratorMut<'a, T> where T : Regular {
        fn source(&self) -> &T {
            unsafe {&*self.ptr}
        }
    }

    impl<'a, T> Writable for SliceIteratorMut<'a, T> where T : Regular {
        fn sink(&self) -> &mut T {
            let v : &mut T;
            unsafe {v = &mut *self.ptr;}
            v
        }
    }

    impl<'a, T> Mutable for SliceIteratorMut<'a, T> where T : Regular {
        fn deref(&self) -> &mut T {
            let v : &mut T; 
            unsafe {v = &mut *self.ptr;}
            v
        }
    }

    impl<'a, T> Iterator for SliceIteratorMut<'a, T>
    where SliceIteratorMut<'a, T> : PartialEq, T : Regular {
        type DistanceType = usize;
        fn successor_assign(&mut self) {
            unsafe {self.ptr = self.ptr.offset(1)};
        }
        fn add_assign(&mut self, n : Self::DistanceType) {
            let m : isize = num::NumCast::from(n).unwrap();
            unsafe {self.ptr = self.ptr.offset(m)};
        }
        fn dif(&self, f : Self) -> <Self as Iterator>::DistanceType {
            num::NumCast::from(
                (self.ptr as usize - f.ptr as usize) / mem::size_of::<T>()
            ).unwrap()
        }
    }

    // This iterator is copyable.
    impl<'a, T> ForwardIterator for SliceIteratorMut<'a, T>
    where SliceIteratorMut<'a, T> : Iterator + Regular {}

    // This iterator provides efficient add and sub operators.
    impl<'a, T> IndexedIterator for SliceIteratorMut<'a, T>
    where SliceIteratorMut<'a, T> : ForwardIterator {}

    // This iterator is bidirectional.
    impl<'a, T> BidirectionalIterator for SliceIteratorMut<'a, T>
    where SliceIteratorMut<'a, T> : ForwardIterator {
        fn predecessor_assign(&mut self) {
            unsafe {self.ptr = self.ptr.offset(-1)}
        }
        fn sub_assign(&mut self, n : Self::DistanceType) {
            let m : isize = num::NumCast::from(n).unwrap();
            unsafe {self.ptr = self.ptr.offset(-m)}
        }
    }

    impl<'a, T> RandomAccessIterator for SliceIteratorMut<'a, T>
    where SliceIteratorMut<'a, T> : IndexedIterator + BidirectionalIterator {
        type DifferenceType = isize;
        fn less_than(&self, y : &Self) -> bool {
            (self.ptr as usize) < (y.ptr as usize)
        }
    }

    //-----------------------------------------------------------------------------
    // Test Slice Iterator
    
    fn test_for_each<I>(f : I, l : &I, j : I::ValueType, k : I::ValueType)
    where I : Readable + Iterator, <I as Reference>::ValueType : AddAssign<I::ValueType> + Debug {
        let mut s : I::ValueType = j;
        for_each(f, l, |v| s += (*v).clone());
        assert_eq!(s, k);
    }

    fn test_find<I>(f : I, l : &I, i : I::ValueType, j : I::ValueType, k : I::ValueType)
    where I : Readable + Iterator, <I as Reference>::ValueType : AddAssign<I::ValueType> + Debug {
        let mut s : I::ValueType = i;
        for_each(find(f, l, j), l, |v| s += (*v).clone());
        assert_eq!(s, k);
    } 

    fn test_find_if<I>(f : I, l : &I, i : I::ValueType, j : I::ValueType, k : I::ValueType)
    where I : Readable + Iterator, <I as Reference>::ValueType : AddAssign<I::ValueType> + Debug { 
        let mut s : I::ValueType = i;
        for_each(find_if(f, l, |v| *v==j), l, |v| s += (*v).clone());
        assert_eq!(s, k);
    }

    fn test_count_if<I>(f : I, l : &I, i : I::ValueType, j : I::ValueType,
        k : I::ValueType, m : I::ValueType)
    where I : Readable + Iterator, I::ValueType : Integer + Debug {
        let c1 : I::ValueType = count_if(f, l, |v| *v > i && *v < j, k);
        assert_eq!(c1, m);
    }

    fn test_count_if_from_zero<I>(f : I, l : &I, i : I::ValueType,
        j : I::ValueType, k : I::DistanceType)
    where I : Readable + Iterator, I::ValueType: PartialOrd, I::DistanceType : Debug {
        let c1 : I::DistanceType = count_if_from_zero(f, l, |v| *v > i && *v < j);
        assert_eq!(c1, k);
    }

    fn test_reduce_nonempty<I>(f : I, l : &I, k : I::ValueType)
    where I : Readable + Iterator, <I as Reference>::ValueType: Debug + Add<Output = I::ValueType> {
        let r = reduce_nonempty(f, l, |a, b| a + b, |a| (*a.source()).clone());
        assert_eq!(r, k);
    }

    fn test_reduce<I>(f : I, l : &I, i : I::ValueType, j : I::ValueType)
    where I : Iterator + Readable, <I as Reference>::ValueType : Add<Output = I::ValueType> + Debug {
        let r = reduce(f, l, |a, b| a + b, |a| (*a.source()).clone(), i);
        assert_eq!(r, j);
    }

    fn test_reduce_nonzeroes<I>(f : I, l : &I, i : I::ValueType, j : I::ValueType)
    where I : Readable + Iterator, <I as Reference>::ValueType : Add<Output = I::ValueType> + Debug {
        let r = reduce_nonzeroes(f, l, |a, b| a + b, |a| (*a.source()).clone(), i);
        assert_eq!(r, j);
    }

    fn test_for_each_n<I>(f : I, n : I::DistanceType, j : I::ValueType, k : I::ValueType)
    where I : Readable + Iterator, <I as Reference>::ValueType : AddAssign<I::ValueType> + Debug {
        let mut s : I::ValueType = j;
        for_each_n(f, n, |v| s += (*v).clone());
        assert_eq!(s, k);
    }

    fn test_find_n<I>(f : I, n : I::DistanceType, i : I::ValueType,
        j : I::ValueType, k : I::ValueType)
    where I : Readable + Iterator, <I as Reference>::ValueType : AddAssign<I::ValueType> + Debug {
        let mut s : I::ValueType = i;
        let (g, m) = find_n(f, n, j);
        for_each_n(g, m, |v| s += (*v).clone());
        assert_eq!(s, k);
    }

    fn test_find_if_unguarded<I>(f : I, l : &I, i : I::ValueType, j : I::ValueType,
        k : I::ValueType)
    where I : Readable + Iterator, <I as Reference>::ValueType : AddAssign<I::ValueType> + Debug {
        let mut s : I::ValueType = i;
        for_each(find_if_unguarded(f, |v| *v==j), l, |v| s += (*v).clone());
        assert_eq!(s, k);
    }

    fn test_find_mismatch<I>(f0 : I, l0 : &I, f1 : I, l1 : &I)
    where I : Readable + Iterator, I : Debug, I::ValueType : Debug {
        let (i,j) = find_mismatch(f0, l0, f1, l1, |a, b| a == b);
        assert!(*i.source() != *j.source());
    }

    fn test_find_adjacent_mismatch<I>(f : I, l : &I, m : &I)
    where I : Readable + Iterator, I::ValueType : Debug {
        let i = find_adjacent_mismatch(f, l, |a, b| a == b);
        assert!(i.source() != (*m).source());
    }

    fn test_relation_preserving<I>(f : I, l : &I)
    where I : Readable + Iterator, I::ValueType : PartialOrd {
        let b = relation_preserving(f, l, |a, b| b > a);
        assert!(b);
    }

    fn test_strictly_increasing_range<I>(f : I, l : &I)
    where I : Readable + Iterator, I::ValueType : PartialOrd {
        let b = strictly_increasing_range(f, l, |a, b| b > a);
        assert!(b);
    }

    fn test_complement_of_converse() {
        let mut r = complement_of_converse(|a, b| b > a);
        assert!(r(&3, &4) == true);
        assert!(r(&4, &4) == true);
        assert!(r(&5, &4) == false);
    }

    fn test_increasing_range<I>(f : I, l : &I)
    where I : Readable + Iterator, I::ValueType : PartialOrd {
        let b = increasing_range(f, l, |a, b| b > a);
        assert!(b);
    }

    fn test_partitioned<I>(f : I, l : &I, x : I::ValueType) 
    where I : Readable + Iterator, I::ValueType : PartialOrd {
        let b = partitioned(f, l, |a| a > &x);
        assert!(b);
    }

    fn test_find_adjacent_mismatch_forward<I>(f : I, l : &I, m : &I)
    where I : Readable + ForwardIterator {
        let i = find_adjacent_mismatch_forward(f, l, |a, b| a == b);
        assert!(i.source() != (*m).source());
    }

    fn test_partition_point<I>(f : I, l : &I, p : I::ValueType, q : I::DistanceType)
    where I : Readable + ForwardIterator, I::DistanceType : PartialEq {
        let i = partition_point(f.clone(), l.clone(), |a| a == &p);
        assert!(l.clone().dif(i) == q);
    }

    fn test_lower_bound_n<I>(f : I, n : I::DistanceType, x : I::ValueType, y : I::ValueType)
    where I : Readable + ForwardIterator, I::ValueType : PartialOrd,
    I::DistanceType : PartialEq {
        let i = lower_bound_n(f.clone(), n, &x, |a, b| a < b);
        assert!(i.source() == &y);
    }

    fn test_upper_bound_n<I>(f : I, n : I::DistanceType, x : I::ValueType, y : I::ValueType)
    where I : Readable + ForwardIterator, I::ValueType : PartialOrd,
    I::DistanceType : PartialEq {
        let i = upper_bound_n(f.clone(), n, &x, |a, b| a < b);
        assert!(i.source() == &y);
    }

    fn test_find_backward_if<I>(f : &I, l : I, i : I::ValueType, j : I::ValueType, k : I::ValueType)
    where I : Readable + BidirectionalIterator, <I as Reference>::ValueType : AddAssign<I::ValueType> + Debug {
        let mut s : I::ValueType = i;
        for_each(find_backward_if(f, l.clone(), |v| *v==j), &l, |v| s += (*v).clone());
        assert_eq!(s, k);
    }

    #[test]
    fn test_iterators() {
        let mut v = [0, 1, 2, 3];
        let mut w = [0, 1, 3, 2];
        let f = v.begin();
        let g = w.begin();
        let l = v.end();
        let m = w.end();
        assert_eq!(v.len(), l.clone().dif(f.clone()));
        assert_eq!(w.len(), m.clone().dif(g.clone()));

        test_for_each(f.clone(), &l, 0, 6);
        test_find(f.clone(), &l, 0, 2, 5);
        test_find_if(f.clone(), &l, 0, 1, 6);
        test_count_if(f.clone(), &l, 0, 3, 1, 3);
        test_count_if_from_zero(f.clone(), &l, 0, 3, 2);
        test_reduce_nonempty(f.clone(), &l, 6);
        test_reduce(f.clone(), &l, 0, 6);
        test_reduce(f.clone(), &f, 0, 0);
        test_reduce_nonzeroes(f.clone(), &l, 0, 6);
        test_for_each_n(f.clone(), v.len(), 0, 6);
        test_find_n(f.clone(), v.len(), 0, 2, 5);
        test_find_if_unguarded(f.clone(), &l, 0, 2, 5);
        test_find_mismatch(f.clone(), &l, g.clone(), &m);
        test_find_adjacent_mismatch(f.clone(), &l, &f);
        test_relation_preserving(f.clone(), &l);
        test_strictly_increasing_range(f.clone(), &l);
        test_complement_of_converse();
        test_increasing_range(f.clone(), &l);
        test_partitioned(f.clone(), &l, 2);
        test_find_adjacent_mismatch_forward(f.clone(), &l, &f);
        test_partition_point(f.clone(), &l, 2, 2);
        test_lower_bound_n(f.clone(), v.len(), 1, 1);
        test_upper_bound_n(f.clone(), v.len(), 1, 2);
        test_find_backward_if(&f, l.clone(), 0, 2, 5);
    }

    /*
    #[test]
    fn test_reverse() {
        let mut v = [1, 2, 3, 4];
        let f = v.begin_mut();
        let l = v.end_mut();
        reverse_bidirectional(f.clone(), l.clone());
        let w = [4, 3, 2, 1];
        let g = w.begin();
        let m = w.end();
        assert!(lexicographical_equal(f, &l, g, &m));
    }
    */
}

