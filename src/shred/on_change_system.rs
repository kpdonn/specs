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

pub trait ChangeAdapter<ChangeType: Event> {
    fn track_changed(&mut self) -> ReaderId<ChangeType>;
    fn populate_changed(&self, reader_id: &mut ReaderId<ChangeType>, value: &mut BitSet);
}

impl<'a, T> ChangeAdapter<InsertedFlag> for WriteStorage<'a, T>
where
    T: Component,
    T::Storage: Tracked,
{
    fn track_changed(&mut self) -> ReaderId<InsertedFlag> {
        self.track_inserted()
    }

    fn populate_changed(&self, reader_id: &mut ReaderId<InsertedFlag>, value: &mut BitSet) {
        self.populate_inserted(reader_id, value)
    }
}

impl<'a, T> ChangeAdapter<ModifiedFlag> for WriteStorage<'a, T>
where
    T: Component,
    T::Storage: Tracked,
{
    fn track_changed(&mut self) -> ReaderId<ModifiedFlag> {
        self.track_modified()
    }

    fn populate_changed(&self, reader_id: &mut ReaderId<ModifiedFlag>, value: &mut BitSet) {
        self.populate_modified(reader_id, value);
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

impl<'a, T, Target, ChangeType: Event, SysData: SystemData<'a>> System<'a> for T
where
    T: OnChangesSystem<'a, Target = Target, SysData = SysData, ChangeType = ChangeType>,
    Target: Component,
    Target::Storage: Tracked,
    for<'all> ReadStorage<'all, Target>: ChangeAdapter<ChangeType>,
    for<'all> WriteStorage<'all, Target>: ChangeAdapter<ChangeType>,
{
    type SystemData = (
        WriteExpect<'a, TrackData<Target, ChangeType>>,
        ReadStorage<'a, Target>,
        SysData,
    );

    fn run(&mut self, (mut data, changes, otherData): Self::SystemData) {
        let reader: *mut ReaderId<ChangeType> = &mut data.reader;
        let dirty = &mut data.dirty;

        unsafe {
            changes.populate_changed(&mut *reader, dirty);
        }

        self.run_with_changed(dirty, otherData);
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        let data: TrackData<Target, ChangeType>;
        {
            let mut storage: WriteStorage<Target> = SystemData::fetch(res);
            data = TrackData::new(storage.track_changed());
        }
        assert!(!res.has_value::<TrackData<Target, ChangeType>>());
        res.insert(data);
    }
}
