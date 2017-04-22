ENTRY(start)

SECTIONS {
	. = 1M;

	.boot :
	{
		KEEP(*(.multiboot_header))
	}

	.text :
	{
		*(.text .text.*)
	}

	.rodata : 
	{
		*(.rodata .rodata.*)
	}

	
}