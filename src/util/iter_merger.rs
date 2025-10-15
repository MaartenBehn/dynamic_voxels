pub enum IM2<T, A, B>
    where
        A: Iterator<Item = T>,
        B: Iterator<Item = T>,
    {
    A(A),
    B(B),
}

impl<T, A, B> Iterator for IM2<T, A, B>
    where
        A: Iterator<Item = T>,
        B: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IM2::A(i) => i.next(),
            IM2::B(i) => i.next(),
        }
    }
}

pub enum IM3<T, A, B, C>
    where
        A: Iterator<Item = T>,
        B: Iterator<Item = T>,
        C: Iterator<Item = T>,
    {
    A(A),
    B(B),
    C(C),
}

impl<T, A, B, C> Iterator for IM3<T, A, B, C>
    where
        A: Iterator<Item = T>,
        B: Iterator<Item = T>,
        C: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IM3::A(i) => i.next(),
            IM3::B(i) => i.next(),
            IM3::C(i) => i.next(),
        }
    }
}

pub enum IM4<T, A, B, C, D>
    where
        A: Iterator<Item = T>,
        B: Iterator<Item = T>,
        C: Iterator<Item = T>,
        D: Iterator<Item = T>,
    {
    A(A),
    B(B),
    C(C),
    D(D),
}

impl<T, A, B, C, D> Iterator for IM4<T, A, B, C, D>
    where
        A: Iterator<Item = T>,
        B: Iterator<Item = T>,
        C: Iterator<Item = T>,
        D: Iterator<Item = T>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IM4::A(i) => i.next(),
            IM4::B(i) => i.next(),
            IM4::C(i) => i.next(),
            IM4::D(i) => i.next(),
        }
    }
}

pub enum IM5<T, A, B, C, D, E>
    where
        A: Iterator<Item = T>,
        B: Iterator<Item = T>,
        C: Iterator<Item = T>,
        D: Iterator<Item = T>,
        E: Iterator<Item = T>,
    {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
}

impl<T, A, B, C, D, E> Iterator for IM5<T, A, B, C, D, E>
    where
        A: Iterator<Item = T>,
        B: Iterator<Item = T>,
        C: Iterator<Item = T>,
        D: Iterator<Item = T>,
        E: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IM5::A(i) => i.next(),
            IM5::B(i) => i.next(),
            IM5::C(i) => i.next(),
            IM5::D(i) => i.next(),
            IM5::E(i) => i.next(),
        }
    }
}


