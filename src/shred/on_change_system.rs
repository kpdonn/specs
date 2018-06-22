use prelude::*;
use shrev::Event;
use shrev::ReaderId;
use std::marker::PhantomData;
use storage::*;
pub trait TrackedComponent: Component {}

impl<T> TrackedComponent for T
where
    T: Component,
    T::Storage: Tracked,
{
}

pub trait OnChangesSystem<'a> {
    type Target: TrackedComponent;
    type SysData: SystemData<'a>;
    type ChangeType: Event;

    fn run_with_changed(&mut self, changed: &BitSet, data: Self::SysData);
}

pub trait ReadChangeAdapter<ChangeType: Event> {
    fn populate_changed(&self, reader_id: &mut ReaderId<ChangeType>, value: &mut BitSet);
}

pub trait WriteChangeAdapter<ChangeType: Event> {
    fn track_changed(&mut self) -> ReaderId<ChangeType>;
}

impl<'a, T> ReadChangeAdapter<InsertedFlag> for ReadStorage<'a, T>
where
    T: Component,
    T::Storage: Tracked,
{
    fn populate_changed(&self, reader_id: &mut ReaderId<InsertedFlag>, value: &mut BitSet) {
        self.populate_inserted(reader_id, value)
    }
}

impl<'a, T> WriteChangeAdapter<InsertedFlag> for WriteStorage<'a, T>
where
    T: Component,
    T::Storage: Tracked,
{
    fn track_changed(&mut self) -> ReaderId<InsertedFlag> {
        self.track_inserted()
    }
}

impl<'a, T> ReadChangeAdapter<ModifiedFlag> for ReadStorage<'a, T>
where
    T: Component,
    T::Storage: Tracked,
{
    fn populate_changed(&self, reader_id: &mut ReaderId<ModifiedFlag>, value: &mut BitSet) {
        self.populate_modified(reader_id, value)
    }
}

impl<'a, T> WriteChangeAdapter<ModifiedFlag> for WriteStorage<'a, T>
where
    T: Component,
    T::Storage: Tracked,
{
    fn track_changed(&mut self) -> ReaderId<ModifiedFlag> {
        self.track_modified()
    }
}

pub struct TrackData<Target: TrackedComponent, ChangeType: Event> {
    reader: ReaderId<ChangeType>,
    dirty: BitSet,
    phantom: PhantomData<*const Target>,
}

impl<T: TrackedComponent, U: Event> TrackData<T, U> {
    fn new(reader: ReaderId<U>) -> TrackData<T, U> {
        TrackData {
            reader,
            dirty: BitSet::new(),
            phantom: PhantomData,
        }
    }
}

unsafe impl<T: TrackedComponent, U: Event> Send for TrackData<T, U> {}
unsafe impl<T: TrackedComponent, U: Event> Sync for TrackData<T, U> {}

impl<'s1, T, X, Y: Event, Z> System<'s1> for T
where
    T: for<'all1> OnChangesSystem<'all1, Target = X, ChangeType = Y, SysData = Z>,
    Z: for<'all2> SystemData<'all2>,
    X: Component,
    X::Storage: Tracked,
    for<'all3> ReadStorage<'all3, X>: ReadChangeAdapter<Y>,
    for<'all4> WriteStorage<'all4, X>: WriteChangeAdapter<Y>,
{
    type SystemData = (WriteExpect<'s1, TrackData<X, Y>>, ReadStorage<'s1, X>, Z);

    fn run(&mut self, (mut data, changes, otherData): Self::SystemData) {
        let reader: *mut ReaderId<Y> = &mut data.reader;
        let dirty = &mut data.dirty;

        unsafe {
            changes.populate_changed(&mut *reader, dirty);
        }

        self.run_with_changed(dirty, otherData);
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        let data: TrackData<X, Y>;
        {
            let mut storage: WriteStorage<X> = SystemData::fetch(res);
            data = TrackData::new(storage.track_changed());
        }
        assert!(!res.has_value::<TrackData<X, Y>>());
        res.insert(data);
    }
}
