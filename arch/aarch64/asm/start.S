// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

#include <asm/utils.S.h>

#include <section_names.h>

/// Initial value for the SCR_EL3 register.
///
/// This sets following bits of the register:
///
/// - `[0]:   SCR_EL3.NS`,   configures lower ELs as non-secure
/// - `[5:4]: SCR_EL3.RES1`, reserved, should be 1
/// - `[10]:  SCR_EL3.RW`,   sets EL2 to AArch64 execution state
#define SCR_EL3_INITIALIZER (0x431)

/// Initial value for the HCR_EL2 register.
///
/// This sets following bits of the register:
///
/// - `[31]: HCR_EL2.RW`, sets EL1 to AArch64 execution state
#define HCR_EL2_INITIALIZER (0x80000000)

/// Initial value for the SPSR_EL3 or SPSR_EL2 registers.
///
/// This sets following bits of the register:
///
/// - `[2] & [0]: SPSR_ELX.M`, configures mode to EL1 using SP_EL1
/// - `[6]:       SPSR_ELX.F`, disables fast interrupts
/// - `[7]:       SPSR_ELX.I`, disables standard interrupts
/// - `[8]:       SPSR_ELX.A`, disables system errors
/// - `[9]:       SPSR_ELX.D`, disables breakpoints
#define SPSR_ELX_INITIALIZER (0x3C5)

/// Initial value for the CPACR_EL1 register.
///
/// This sets following bits of the register:
///
/// - `[21:20]: CPACR_EL1.FPEN`, Disables trapping FP and SIMD instructions
#define CPACR_EL1_INITIALIZER (0x300000)

/// @fn    start
/// @brief Entry point of the bootloader.
///
/// TODO: elaborate
SECTION(SNAME_START_TEXT)
BEGIN_FUNCTION(start)
	// Check current core within a processor and stall all except the
	// primary one for the initial setup phases.
	MRS	x9, mpidr_el1
	AND	x9, x9, #0xFF   // Mask affinity 0 bits
	CBZ	x9, drop_to_el1
0:	WFE
	B	0b

	// Get current exception level, drop to EL1 if necessary.
drop_to_el1:
	MRS	x9, currentel
	AND	x9, x9, #0x6
	LSR	x9, x9, #2
	CMP	x9, #3
	B.EQ	from_el3
	CMP	x9, #2
	B.EQ	from_el2
	B	el1_entry

	// Drop to EL1 from EL3.
from_el3:
	MOV	x9, #(SCR_EL3_INITIALIZER)
	MSR	scr_el3, x9
	MOV	x9, #(SPSR_ELX_INITIALIZER)
	MSR	spsr_el3, x9
	ADR	x9, el1_entry
	MSR	elr_el3, x9
	ERET

	// Drop to EL1 from EL2.
from_el2:
	MOV	x9, #(HCR_EL2_INITIALIZER)
	MSR	hcr_el2, x9
	MOV	x9, #(SPSR_ELX_INITIALIZER)
	MSR	spsr_el2, x9
	ADR	x9, el1_entry
	MSR	elr_el2, x9
	ERET

el1_entry:
	// Set stack pointer.
	ADR	x9, __estack
	MOV	sp, x9

	// Enable full access to floating point and SIMD for EL1 and EL0.
	MOV	x9, #(CPACR_EL1_INITIALIZER)
	MSR	cpacr_el1, x9
	ISB

	// Populate .bss section with zeros.
	ADR	x9, __bss
	ADR	x10, __ebss
0:	CMP	x9, x10
	B.GE	branch_to_hll
	STP	xzr, xzr, [x9]
	ADD	x9, x9, #16
	B	0b

branch_to_hll:
	BL	kmain
	LDR	x0, =0xC0DEDEAD
0:	B	0b
END_FUNCTION(start)
