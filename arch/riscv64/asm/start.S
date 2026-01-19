// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

	.section .text.start
	.globl start

start:
	LI	t0, 0xABCD
1:	J	1b
