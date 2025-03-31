MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /*SPM : ORIGIN = 0x00000000, LENGTH = 320K*/
  FLASH : ORIGIN = 0x00000000, LENGTH = 1024K
  RAM   : ORIGIN = 0x20010000, LENGTH = 192K - 1K - 1K
  PANDUMP : ORIGIN = ORIGIN(RAM) + LENGTH(RAM), LENGTH = 1K
  PERSISTENT_BUFF : ORIGIN = ORIGIN(PANDUMP) + LENGTH(PANDUMP), LENGTH = 1K
  IPC   : ORIGIN = 0x20000000, LENGTH = 64K
}

/* _config = ORIGIN(CONFIG); */

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* You may want to use this variable to locate the call stack and static
   variables in different memory regions. Below is shown the default value */
/* _stack_start = ORIGIN(RAM) + LENGTH(RAM); */

/* You can use this symbol to customize the location of the .text section */
/* If omitted the .text section will be placed right after the .vector_table
   section */
/* This is required only on microcontrollers that store some configuration right
   after the vector table */
/* _stext = ORIGIN(FLASH) + 0x400; */

/* Example of putting non-initialized variables into custom RAM locations. */
/* This assumes you have defined a region RAM2 above, and in the Rust
   sources added the attribute `#[link_section = ".ram2bss"]` to the data
   you want to place there. */
/* Note that the section will not be zero-initialized by the runtime! */
/*SECTIONS
{
  .spm :
  {
    KEEP(*(.spm .spm.*));
  } > SPM
}*/

_panic_dump_start = ORIGIN(PANDUMP);
_panic_dump_end   = ORIGIN(PANDUMP) + LENGTH(PANDUMP);
_persistent_buff_start = ORIGIN(PERSISTENT_BUFF);
_persistent_buff_end   = ORIGIN(PERSISTENT_BUFF) + LENGTH(PERSISTENT_BUFF);

PROVIDE(__start_ipc = ORIGIN(IPC));
PROVIDE(__end_ipc   = ORIGIN(IPC) + LENGTH(IPC));