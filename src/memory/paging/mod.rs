mod entry;
mod table;
mod temporary_page;
mod mapper;

pub use self::entry::*;
pub use self::mapper::Mapper;
use core::ops::{Deref, DerefMut};
use self::temporary_page::TemporaryPage;
use memory::{Frame, PAGE_SIZE, FrameAllocator};
use multiboot2::BootInformation;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub struct ActivePageTable {
	mapper: Mapper
}

impl Deref for ActivePageTable {
	type Target = Mapper;
	fn deref(&self) -> &Mapper {
		&self.mapper
	}
}

impl DerefMut for ActivePageTable {
	fn deref_mut(&mut self) -> &mut Mapper {
		&mut self.mapper
	}
}

impl ActivePageTable {
	unsafe fn new() -> ActivePageTable {
		ActivePageTable {
			mapper: Mapper::new(),
		}
	}

	pub fn with<F>(&mut self, 
		table: &mut InactivePageTable, 
		temporary_page: &mut temporary_page::TemporaryPage,
		f: F)
		where F: FnOnce(&mut Mapper)
	{
		use x86_64::instructions::tlb;
		use x86_64::registers::control_regs;
		{

			let backup = Frame::containing_address(
				control_regs::cr3().0 as usize
			);

			let p4_table = temporary_page.map_table_frame(backup.clone(), self);

			// overwrite recursive mapping so that it points to the inactive table
			self.p4_mut()[511].set(table.p4_frame.clone(), PRESENT | WRITABLE);
			tlb::flush_all();
			f(self);

			p4_table[511].set(backup, PRESENT | WRITABLE);
			tlb::flush_all();
		}

		temporary_page.unmap(self);
	}

	pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
		use x86_64::PhysicalAddress;
		use x86_64::registers::control_regs;

		let old_table = InactivePageTable {
			p4_frame: Frame::containing_address(
				control_regs::cr3().0 as usize
			),
		};
		unsafe {
			control_regs::cr3_write(PhysicalAddress(
				new_table.p4_frame.start_address() as u64
			));
		}
		old_table
	}
}

pub struct InactivePageTable {
	p4_frame: Frame,
}

impl InactivePageTable {
	pub fn new(frame: Frame, active_table: &mut ActivePageTable,
		temporary_page: &mut TemporaryPage)
		-> InactivePageTable {
		{
			let table = temporary_page.map_table_frame(frame.clone(), active_table);
			table.zero();
			table[511].set(frame.clone(), PRESENT | WRITABLE);
		}
		temporary_page.unmap(active_table);

		InactivePageTable { p4_frame: frame }
	}
	
}

/// Page of virtual memory
#[derive(Debug, Clone, Copy)]
pub struct Page {
	number: usize,
}

impl Page {
	/// Returns the starting address of the page
	fn start_address(&self) -> usize {
		self.number * PAGE_SIZE
	}

	/// Returns the page index in the P4 table
	fn p4_index(&self) -> usize {
    	(self.number >> 27) & 0o777
	}
	/// Returns the page index in the P3 table
	fn p3_index(&self) -> usize {
    	(self.number >> 18) & 0o777
	}
	/// Returns the page index in the P2 table
	fn p2_index(&self) -> usize {
    	(self.number >> 9) & 0o777
	}
	/// Returns the page index in the P1 table
	fn p1_index(&self) -> usize {
    	(self.number >> 0) & 0o777
	}

	/// Returns the page containing the virtual address
	///
	/// Panics if address is invalid!
	pub fn containing_address(address: VirtualAddress) -> Page {
		assert!(address < 0x0000_8000_0000_0000 ||
			address >= 0xffff_8000_0000_0000, "invalid address 0x{}", address);
		Page { number: address / PAGE_SIZE }
	}
}

pub fn remap_the_kernel<A>(allocator: &mut A, boot_info: &BootInformation)
	where A: FrameAllocator
{
	let mut temporary_page = TemporaryPage::new(Page { number: 0xdeadbeaf }, allocator);
	let mut active_table = unsafe { ActivePageTable::new() };
	let mut new_table = {
		let frame = allocator.allocate_frame().expect("no more frames");
		InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
	};
	active_table.with(&mut new_table, &mut temporary_page, |mapper| {
		let elf_sections_tag = boot_info.elf_sections_tag()
			.expect("Memory map tag required");

		for section in elf_sections_tag.sections() {
			if !section.is_allocated() {
				continue;
			}
			assert!(section.start_address() % PAGE_SIZE == 0,
				"sections needs to be page aligned");
			println!("Mapping section at addr: {:#x}, size: {:#x}",
				section.addr, section.size);
			let flags = EntryFlags::from_elf_section_flags(section);

			let start_frame = Frame::containing_address(section.start_address());
			let end_frame = Frame::containing_address(section.end_address() -1);
			for frame in Frame::range_inclusive(start_frame, end_frame) {
				mapper.identity_map(frame, flags, allocator);
			}
		}

		// Identity map the VGA buffer
		let vga_buffer_frame = Frame::containing_address(0xb8000);
		mapper.identity_map(vga_buffer_frame, WRITABLE, allocator);

		// Identity map the multiboot info structure
		let multiboot_start = Frame::containing_address(boot_info.start_address());
		let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
		for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
			mapper.identity_map(frame, PRESENT, allocator);
		}
	});
	let old_table = active_table.switch(new_table);
	println!("Swaping Table!");

	let old_p4_page = Page::containing_address(
		old_table.p4_frame.start_address()
	);
	active_table.unmap(old_p4_page, allocator);
	println!("guard page at {:#x}", old_p4_page.start_address());
}


