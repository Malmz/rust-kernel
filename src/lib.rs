#![feature(lang_items, const_fn, unique)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;

#[macro_use]
mod vga_buffer;

#[no_mangle]
pub extern fn rust_main(multiboot_information_adress: usize) {
	let boot_info = unsafe{ multiboot2::load(multiboot_information_adress) };
	vga_buffer::clear_screen();
	
	let memory_map_tag = boot_info.memory_map_tag()
		.expect("Memory map tag required");
	
	println!("Memory areas:");
	for area in memory_map_tag.memory_areas() {
		println!("\tstart: 0x{:x}, length: 0x{:x}",
			area.base_addr, area.length);
	}

	let elf_sections_tag = boot_info.elf_sections_tag()
		.expect("Elf-sections tag required");
	println!("Kernel sections:");
	for sections in elf_sections_tag.sections() {
		println!("\taddr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
			sections.addr, sections.size, sections.flags);
	}

	let kernel_start = elf_sections_tag.sections().map(|s| s.addr)
		.min().unwrap();
	let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size)
		.max().unwrap();
	let multiboot_start = multiboot_information_adress;
	let multiboot_end = multiboot_start + (boot_info.total_size as usize);

	loop{}
}

#[lang = "eh_personality"] extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
	println!("\n\nPANIC in {} at line {}:", file, line);
	println!("\t{}", fmt);
	loop{}
}