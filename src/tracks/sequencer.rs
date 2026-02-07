use super::*;

#[derive(Debug, Clone)]
pub struct Step {
    pub id: usize,
    pub trig_type: TrigType,
    pub note: Option<u8>,
    pub velocity: f32,
    pub param_locks: ParamLocks,
    pub micro_timing: i8,
    pub retrig: bool,
    pub retrig_rate: RetrigRate,
    pub retrig_vel_curve: f32,
    pub condition: Option<TrigCondition>,
}

impl Default for Step {
    fn default() -> Self {
        Step {
            id: 0,
            trig_type: TrigType::None,
            note: None,
            velocity: 100.0,
            param_locks: ParamLocks::new(),
            micro_timing: 0,
            retrig: false,
            retrig_rate: RetrigRate::Quarter,
            retrig_vel_curve: 0.0,
            condition: None,
        }
    }
}

impl Step {
    pub fn has_trig(&self) -> bool {
        self.trig_type != TrigType::None
    }

    pub fn is_note_trig(&self) -> bool {
        self.trig_type == TrigType::Note || self.trig_type == TrigType::Combined
    }

    pub fn is_lock_trig(&self) -> bool {
        self.trig_type == TrigType::Lock || self.trig_type == TrigType::Combined
    }
}

#[derive(Debug, Clone)]
pub struct ParamLocks(pub Vec<(String, f32)>);

impl ParamLocks {
    pub fn new() -> Self {
        ParamLocks(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        ParamLocks(Vec::with_capacity(capacity))
    }

    pub fn get(&self, param: &str) -> Option<f32> {
        self.0.iter().find(|(p, _)| p == param).map(|(_, v)| *v)
    }

    pub fn set(&mut self, param: String, value: f32) {
        if let Some(existing) = self.0.iter_mut().find(|(p, _)| *p == param) {
            existing.1 = value;
        } else {
            self.0.push((param, value));
        }
    }

    pub fn remove(&mut self, param: &str) {
        self.0.retain(|(p, _)| p != param);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Debug, Clone)]
pub struct SequencerPage {
    pub id: usize,
    pub steps: [Step; STEPS_PER_PAGE],
    pub length: u8,
    pub scale: f32,
}

impl Default for SequencerPage {
    fn default() -> Self {
        let steps = core::array::from_fn(|i| Step {
            id: i,
            trig_type: TrigType::None,
            note: None,
            velocity: 100.0,
            param_locks: ParamLocks(Vec::new()),
            micro_timing: 0,
            retrig: false,
            retrig_rate: RetrigRate::Quarter,
            retrig_vel_curve: 0.0,
            condition: None,
        });
        SequencerPage {
            id: 0,
            steps,
            length: 16,
            scale: 1.0,
        }
    }
}

impl SequencerPage {
    pub fn new(id: usize) -> Self {
        let mut page = SequencerPage::default();
        page.id = id;
        page
    }

    pub fn active_steps(&self) -> usize {
        self.length as usize
    }

    pub fn step(&self, index: usize) -> Option<&Step> {
        if index < self.length as usize {
            Some(&self.steps[index])
        } else {
            None
        }
    }

    pub fn step_mut(&mut self, index: usize) -> Option<&mut Step> {
        if index < self.length as usize {
            Some(&mut self.steps[index])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrackSequencer {
    pub enabled: bool,
    pub length: u8,
    pub scale: f32,
    pub pages: [SequencerPage; PAGES],
    pub swing: f32,
    pub pattern_length: u8,
    pub change: bool,
    pub reset: bool,
}

impl Default for TrackSequencer {
    fn default() -> Self {
        let pages = core::array::from_fn(|i| SequencerPage {
            id: i,
            steps: core::array::from_fn(|j| Step {
                id: j,
                trig_type: TrigType::None,
                note: None,
                velocity: 100.0,
                param_locks: ParamLocks(Vec::new()),
                micro_timing: 0,
                retrig: false,
                retrig_rate: RetrigRate::Quarter,
                retrig_vel_curve: 0.0,
                condition: None,
            }),
            length: 16,
            scale: 1.0,
        });
        TrackSequencer {
            enabled: false,
            length: 16,
            scale: 1.0,
            pages,
            swing: 0.0,
            pattern_length: 16,
            change: false,
            reset: false,
        }
    }
}

impl TrackSequencer {
    pub fn new() -> Self {
        TrackSequencer::default()
    }

    pub fn total_steps(&self) -> usize {
        (self.length * self.pattern_length) as usize
    }

    pub fn step(&self, global_step: usize) -> Option<&Step> {
        let page_index = global_step / STEPS_PER_PAGE;
        let step_index = global_step % STEPS_PER_PAGE;
        if page_index < PAGES {
            self.pages[page_index].step(step_index)
        } else {
            None
        }
    }

    pub fn step_mut(&mut self, global_step: usize) -> Option<&mut Step> {
        let page_index = global_step / STEPS_PER_PAGE;
        let step_index = global_step % STEPS_PER_PAGE;
        if page_index < PAGES {
            self.pages[page_index].step_mut(step_index)
        } else {
            None
        }
    }

    pub fn trig_steps(&self, global_step: usize) -> Vec<&Step> {
        let mut trigs = Vec::new();
        let page_index = global_step / STEPS_PER_PAGE;
        let step_index = global_step % STEPS_PER_PAGE;

        if page_index < PAGES {
            let page = &self.pages[page_index];
            for i in 0..page.length as usize {
                if let Some(step) = page.step(i) {
                    if step.has_trig() {
                        let offset = step.micro_timing as f32 / 127.0;
                        if offset.abs() < 0.1 {
                            trigs.push(step);
                        }
                    }
                }
            }
        }
        trigs
    }

    pub fn set_page_length(&mut self, page_id: usize, length: u8) {
        if page_id < PAGES && length >= 1 && length <= 16 {
            self.pages[page_id].length = length;
        }
    }

    pub fn set_page_scale(&mut self, page_id: usize, scale: f32) {
        if page_id < PAGES && scale >= 0.125 && scale <= 2.0 {
            self.pages[page_id].scale = scale;
        }
    }
}
