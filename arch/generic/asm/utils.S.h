// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

#pragma once

#if defined(__ASSEMBLER__)

  /// @defgroup ElfTypes ELF section types
  /// @{
  #define SHT_PROGBITS %progbits ///< Section contents are program-defined.
  #define SHT_NOBITS   %nobits   ///< Section occupies no size on disk.
  /// @}

  /// @defgroup ElfAttributes ELF section attribute constants
  /// @{
  #define SHF_WRITE     "w" ///< Section is writable.
  #define SHF_ALLOC     "a" ///< Section is allocatable.
  #define SHF_EXECINSTR "x" ///< Section is executable.
  /// @}

  /// Assemble following code into a particular section.
  /// @param name Name of the section.
  ///
  /// @see SECTION3
  #define SECTION1(name) \
  	.section name

  /// Assemble following code into a particular section.
  /// @param name Name of the section.
  /// @param type One of constants from @ref ElfTypes.
  /// @param attr One of constants from @ref ElfAttributes.
  ///
  /// @see SECTION1
  #define SECTION3(name, type, attr) \
  	.section name, attr, type

  /// Declare a beginning of a global function.
  /// @param name Symbol for the function.
  ///
  /// @see END_FUNCTION
  #define BEGIN_FUNCTION(name)   \
  	.globl	name;            \
  	.type	name, %function; \
  name:;

  /// Declare end of a function.
  /// @param name Symbol of the function.
  ///
  /// @see BEGIN_FUNCTION
  #define END_FUNCTION(name) \
  	.size	name, .-name

#endif // defined(__ASSEMBLER__)
