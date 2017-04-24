section .multiboot_header
header_start:
	dd 0xe85250d6 ; Magic number
	dd 0 ; Protected mode code
	dd header_end - header_start ; header length

	; Checksum
	dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

	; End data tags
	dw 0 ; type
	dw 0 ; flags
	dd 8 ; size
header_end: