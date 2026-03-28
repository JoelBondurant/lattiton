use iced::Size;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaneId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
	Horizontal,
	Vertical,
}

impl Axis {
	pub fn is_horizontal(self) -> bool {
		self == Axis::Horizontal
	}

	pub fn split_size(self, size: Size) -> (f32, f32) {
		match self {
			Axis::Horizontal => (size.width, size.height),
			Axis::Vertical => (size.height, size.width),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollapseState {
	Expanded,
	FirstCollapsed,
	SecondCollapsed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitId(pub usize);

#[derive(Debug, Clone)]
pub struct Split {
	pub axis: Axis,
	pub ratio: f32,
	pub saved_ratio: f32,
	pub collapse: CollapseState,
	pub first: NodeId,
	pub second: NodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeId {
	Split(SplitId),
	Pane(PaneId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaximizeState {
	None,
	Maximized(PaneId),
}

#[derive(Debug, Clone)]
pub struct State {
	panes: Vec<PaneId>,
	splits: Vec<(SplitId, Split)>,
	root: Option<NodeId>,
	next_pane_id: usize,
	next_split_id: usize,
	maximize: MaximizeState,
	dragging: Option<SplitId>,
}

impl State {
	pub fn new() -> Self {
		Self {
			panes: Vec::new(),
			splits: Vec::new(),
			root: None,
			next_pane_id: 0,
			next_split_id: 0,
			maximize: MaximizeState::None,
			dragging: None,
		}
	}

	pub fn with_initial_pane() -> (Self, PaneId) {
		let mut state = Self::new();
		let pane = state.alloc_pane();
		state.root = Some(NodeId::Pane(pane));
		(state, pane)
	}

	fn alloc_pane(&mut self) -> PaneId {
		let id = PaneId(self.next_pane_id);
		self.next_pane_id += 1;
		self.panes.push(id);
		id
	}

	fn alloc_split(&mut self, split: Split) -> SplitId {
		let id = SplitId(self.next_split_id);
		self.next_split_id += 1;
		self.splits.push((id, split));
		id
	}

	pub fn split(&mut self, axis: Axis, pane: PaneId) -> Option<(SplitId, PaneId)> {
		let new_pane = self.alloc_pane();
		let node = self.find_node(NodeId::Pane(pane))?;

		let split = Split {
			axis,
			ratio: 0.5,
			saved_ratio: 0.5,
			collapse: CollapseState::Expanded,
			first: NodeId::Pane(pane),
			second: NodeId::Pane(new_pane),
		};

		let split_id = self.alloc_split(split);
		let split_node = NodeId::Split(split_id);

		if self.root == Some(node) {
			self.root = Some(split_node);
		} else {
			self.replace_node(node, split_node);
		}

		Some((split_id, new_pane))
	}

	fn find_node(&self, target: NodeId) -> Option<NodeId> {
		if self.root == Some(target) {
			return Some(target);
		}
		for (_, split) in &self.splits {
			if split.first == target || split.second == target {
				return Some(target);
			}
		}
		None
	}

	fn replace_node(&mut self, old: NodeId, new: NodeId) {
		if self.root == Some(old) {
			self.root = Some(new);
			return;
		}
		for (_, split) in &mut self.splits {
			if split.first == old {
				split.first = new;
				return;
			}
			if split.second == old {
				split.second = new;
				return;
			}
		}
	}

	pub fn get_split(&self, id: SplitId) -> Option<&Split> {
		self.splits.iter().find(|(sid, _)| *sid == id).map(|(_, s)| s)
	}

	pub fn get_split_mut(&mut self, id: SplitId) -> Option<&mut Split> {
		self.splits.iter_mut().find(|(sid, _)| *sid == id).map(|(_, s)| s)
	}

	pub fn root(&self) -> Option<NodeId> {
		self.root
	}

	pub fn panes(&self) -> &[PaneId] {
		&self.panes
	}

	pub fn maximize(&self) -> MaximizeState {
		self.maximize
	}

	pub fn toggle_maximize(&mut self, pane: PaneId) {
		self.maximize = match self.maximize {
			MaximizeState::Maximized(p) if p == pane => MaximizeState::None,
			_ => MaximizeState::Maximized(pane),
		};
	}

	pub fn restore_maximize(&mut self) {
		self.maximize = MaximizeState::None;
	}

	pub fn collapse_first(&mut self, split_id: SplitId) {
		if let Some(split) = self.get_split_mut(split_id) {
			if split.collapse == CollapseState::Expanded {
				split.saved_ratio = split.ratio;
			}
			split.collapse = CollapseState::FirstCollapsed;
			split.ratio = 0.0;
		}
	}

	pub fn collapse_second(&mut self, split_id: SplitId) {
		if let Some(split) = self.get_split_mut(split_id) {
			if split.collapse == CollapseState::Expanded {
				split.saved_ratio = split.ratio;
			}
			split.collapse = CollapseState::SecondCollapsed;
			split.ratio = 1.0;
		}
	}

	pub fn expand(&mut self, split_id: SplitId) {
		if let Some(split) = self.get_split_mut(split_id) {
			split.ratio = split.saved_ratio;
			split.collapse = CollapseState::Expanded;
		}
	}

	pub fn set_dragging(&mut self, split: Option<SplitId>) {
		self.dragging = split;
	}

	pub fn dragging(&self) -> Option<SplitId> {
		self.dragging
	}

	pub fn resize(&mut self, split_id: SplitId, ratio: f32) {
		if let Some(split) = self.get_split_mut(split_id)
		&& split.collapse == CollapseState::Expanded {
			split.ratio = ratio.clamp(0.05, 0.95);
			split.saved_ratio = split.ratio;
		}
	}
}
