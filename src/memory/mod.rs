mod area_frame_allocator;
mod paging;

pub use self::area_frame_allocator::AreaFrameAllocator;
pub use self::paging::test_paging;
use self::paging::PhysicalAddress;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
	number: usize,
}

pub const PAGE_SIZE: usize = 4096;

impl Frame {
	/// Returns the frame containing the virtual adress
	fn containing_address(address: usize) -> Frame {
		Frame { number: address / PAGE_SIZE }
	}

	///Returns the starting physical address of the frame
	fn start_address(&self) -> PhysicalAddress {
		self.number * PAGE_SIZE
	}
}

pub trait FrameAllocator {
	fn allocate_frame(&mut self) -> Option<Frame>;
	fn deallocate_frame(&mut self, frame: Frame);
}
