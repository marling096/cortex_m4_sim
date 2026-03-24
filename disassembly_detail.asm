================================================================================
                    ARM Cortex-M4 反汇编详细信息
                    起始地址: 0x08000000 (Reset_Handler)
================================================================================

┌─────────────────────────────────────────────────────────────────────────────┐
│                              CODE SECTION                                    │
└─────────────────────────────────────────────────────────────────────────────┘


════════════════════════════════════════════════════════════════════════════
FUNCTION: <_main_stk>
════════════════════════════════════════════════════════════════════════════
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080000EC
│ Bytes:      DF F8 0C D0
│ Mnemonic:   ldr.w
│ Op String:  sp, [pc, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: sp
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080000F0
│ Bytes:      00 F0 18 F8
│ Mnemonic:   bl
│ Op String:  #0x8000124
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000124 (134218020)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080000F4
│ Bytes:      00 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080000F6
│ Bytes:      00 47
│ Mnemonic:   bx
│ Op String:  r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: r0
│       Shift: None

════════════════════════════════════════════════════════════════════════════
FUNCTION: <Reset_Handler>
════════════════════════════════════════════════════════════════════════════
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000100
│ Bytes:      06 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0x18]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=24]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000102
│ Bytes:      80 47
│ Mnemonic:   blx
│ Op String:  r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000104
│ Bytes:      06 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0x18]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=24]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000106
│ Bytes:      00 47
│ Mnemonic:   bx
│ Op String:  r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000108
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x8000108
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000108 (134217992)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800010A
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x800010a
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800010A (134217994)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800010C
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x800010c
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800010C (134217996)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800010E
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x800010e
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800010E (134217998)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000110
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x8000110
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000110 (134218000)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000112
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x8000112
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000112 (134218002)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000114
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x8000114
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000114 (134218004)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000116
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x8000116
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000116 (134218006)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000118
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x8000118
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000118 (134218008)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800011A
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x800011a
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800011A (134218010)
│       Shift: None

════════════════════════════════════════════════════════════════════════════
FUNCTION: <__scatterload_rt2>
════════════════════════════════════════════════════════════════════════════
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000124
│ Bytes:      06 4C
│ Mnemonic:   ldr
│ Op String:  r4, [pc, #0x18]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=24]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000126
│ Bytes:      07 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x1c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=28]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000128
│ Bytes:      06 E0
│ Mnemonic:   b
│ Op String:  #0x8000138
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000138 (134218040)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800012A
│ Bytes:      E0 68
│ Mnemonic:   ldr
│ Op String:  r0, [r4, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r4, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800012C
│ Bytes:      40 F0 01 03
│ Mnemonic:   orr
│ Op String:  r3, r0, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000130
│ Bytes:      94 E8 07 00
│ Mnemonic:   ldm.w
│ Op String:  r4, {r0, r1, r2}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (4):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Register: r1
│       Shift: None
│   [3] Type:  Register: r2
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000134
│ Bytes:      98 47
│ Mnemonic:   blx
│ Op String:  r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000136
│ Bytes:      10 34
│ Mnemonic:   adds
│ Op String:  r4, #0x10
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Immediate: 0x10 (16)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000138
│ Bytes:      AC 42
│ Mnemonic:   cmp
│ Op String:  r4, r5
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800013A
│ Bytes:      F6 D3
│ Mnemonic:   blo
│ Op String:  #0x800012a
│ Condition:  ARM_CC_LO
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800012A (134218026)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800013C
│ Bytes:      FF F7 DA FF
│ Mnemonic:   bl
│ Op String:  #0x80000f4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80000F4 (134217972)
│       Shift: None

════════════════════════════════════════════════════════════════════════════
FUNCTION: <BusFault_Handler>
════════════════════════════════════════════════════════════════════════════
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000148
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800014A
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x800014a
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800014A (134218058)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800014C
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800014E
│ Bytes:      00 EB C0 01
│ Mnemonic:   add.w
│ Op String:  r1, r0, r0, lsl #3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Register: r0
│       Shift: LSL #3
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000152
│ Bytes:      C9 00
│ Mnemonic:   lsls
│ Op String:  r1, r1, #3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
│   [2] Type:  Immediate: 0x3 (3)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000154
│ Bytes:      4F F0 E0 22
│ Mnemonic:   mov.w
│ Op String:  r2, #-0x1fff2000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Immediate: 0xFFFFFFFFE000E000 (-536813568)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000158
│ Bytes:      51 61
│ Mnemonic:   str
│ Op String:  r1, [r2, #0x14]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=20]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800015A
│ Bytes:      00 21
│ Mnemonic:   movs
│ Op String:  r1, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800015C
│ Bytes:      91 61
│ Mnemonic:   str
│ Op String:  r1, [r2, #0x18]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=24]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800015E
│ Bytes:      05 21
│ Mnemonic:   movs
│ Op String:  r1, #5
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x5 (5)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000160
│ Bytes:      11 61
│ Mnemonic:   str
│ Op String:  r1, [r2, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000162
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000164
│ Bytes:      4F F0 E0 21
│ Mnemonic:   mov.w
│ Op String:  r1, #-0x1fff2000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0xFFFFFFFFE000E000 (-536813568)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000168
│ Bytes:      09 69
│ Mnemonic:   ldr
│ Op String:  r1, [r1, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800016A
│ Bytes:      01 F4 80 31
│ Mnemonic:   and
│ Op String:  r1, r1, #0x10000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
│   [2] Type:  Immediate: 0x10000 (65536)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800016E
│ Bytes:      00 29
│ Mnemonic:   cmp
│ Op String:  r1, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000170
│ Bytes:      F8 D0
│ Mnemonic:   beq
│ Op String:  #0x8000164
│ Condition:  ARM_CC_EQ
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000164 (134218084)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000172
│ Bytes:      04 21
│ Mnemonic:   movs
│ Op String:  r1, #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x4 (4)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000174
│ Bytes:      4F F0 E0 22
│ Mnemonic:   mov.w
│ Op String:  r2, #-0x1fff2000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Immediate: 0xFFFFFFFFE000E000 (-536813568)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000178
│ Bytes:      11 61
│ Mnemonic:   str
│ Op String:  r1, [r2, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800017A
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800017C
│ Bytes:      2D E9 F0 41
│ Mnemonic:   push.w
│ Op String:  {r4, r5, r6, r7, r8, lr}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (6):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Register: r6
│       Shift: None
│   [3] Type:  Register: r7
│       Shift: None
│   [4] Type:  Register: r8
│       Shift: None
│   [5] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000180
│ Bytes:      02 46
│ Mnemonic:   mov
│ Op String:  r2, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000182
│ Bytes:      00 25
│ Mnemonic:   movs
│ Op String:  r5, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000184
│ Bytes:      00 26
│ Mnemonic:   movs
│ Op String:  r6, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r6
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000186
│ Bytes:      00 20
│ Mnemonic:   movs
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000188
│ Bytes:      00 23
│ Mnemonic:   movs
│ Op String:  r3, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800018A
│ Bytes:      00 24
│ Mnemonic:   movs
│ Op String:  r4, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800018C
│ Bytes:      00 27
│ Mnemonic:   movs
│ Op String:  r7, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r7
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800018E
│ Bytes:      91 F8 03 C0
│ Mnemonic:   ldrb.w
│ Op String:  ip, [r1, #3]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=3]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000192
│ Bytes:      0C F0 0F 05
│ Mnemonic:   and
│ Op String:  r5, ip, #0xf
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Immediate: 0xF (15)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000196
│ Bytes:      91 F8 03 C0
│ Mnemonic:   ldrb.w
│ Op String:  ip, [r1, #3]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=3]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800019A
│ Bytes:      0C F0 10 0C
│ Mnemonic:   and
│ Op String:  ip, ip, #0x10
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Immediate: 0x10 (16)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800019E
│ Bytes:      BC F1 00 0F
│ Mnemonic:   cmp.w
│ Op String:  ip, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001A2
│ Bytes:      03 D0
│ Mnemonic:   beq
│ Op String:  #0x80001ac
│ Condition:  ARM_CC_EQ
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80001AC (134218156)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001A4
│ Bytes:      91 F8 02 C0
│ Mnemonic:   ldrb.w
│ Op String:  ip, [r1, #2]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=2]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001A8
│ Bytes:      4C EA 05 05
│ Mnemonic:   orr.w
│ Op String:  r5, ip, r5
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Register: r5
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001AC
│ Bytes:      91 F8 00 C0
│ Mnemonic:   ldrb.w
│ Op String:  ip, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001B0
│ Bytes:      BC F1 00 0F
│ Mnemonic:   cmp.w
│ Op String:  ip, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001B4
│ Bytes:      31 D0
│ Mnemonic:   beq
│ Op String:  #0x800021a
│ Condition:  ARM_CC_EQ
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800021A (134218266)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001B6
│ Bytes:      14 68
│ Mnemonic:   ldr
│ Op String:  r4, [r2]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001B8
│ Bytes:      00 20
│ Mnemonic:   movs
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001BA
│ Bytes:      2B E0
│ Mnemonic:   b
│ Op String:  #0x8000214
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000214 (134218260)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001BC
│ Bytes:      4F F0 01 0C
│ Mnemonic:   mov.w
│ Op String:  ip, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001C0
│ Bytes:      0C FA 00 F3
│ Mnemonic:   lsl.w
│ Op String:  r3, ip, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001C4
│ Bytes:      B1 F8 00 C0
│ Mnemonic:   ldrh.w
│ Op String:  ip, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001C8
│ Bytes:      0C EA 03 06
│ Mnemonic:   and.w
│ Op String:  r6, ip, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r6
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001CC
│ Bytes:      9E 42
│ Mnemonic:   cmp
│ Op String:  r6, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r6
│       Shift: None
│   [1] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001CE
│ Bytes:      20 D1
│ Mnemonic:   bne
│ Op String:  #0x8000212
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000212 (134218258)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001D0
│ Bytes:      83 00
│ Mnemonic:   lsls
│ Op String:  r3, r0, #2
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x2 (2)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001D2
│ Bytes:      4F F0 0F 0C
│ Mnemonic:   mov.w
│ Op String:  ip, #0xf
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0xF (15)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001D6
│ Bytes:      0C FA 03 F7
│ Mnemonic:   lsl.w
│ Op String:  r7, ip, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r7
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001DA
│ Bytes:      BC 43
│ Mnemonic:   bics
│ Op String:  r4, r7
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r7
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001DC
│ Bytes:      05 FA 03 FC
│ Mnemonic:   lsl.w
│ Op String:  ip, r5, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001E0
│ Bytes:      4C EA 04 04
│ Mnemonic:   orr.w
│ Op String:  r4, ip, r4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Register: r4
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001E4
│ Bytes:      91 F8 03 C0
│ Mnemonic:   ldrb.w
│ Op String:  ip, [r1, #3]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=3]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001E8
│ Bytes:      BC F1 28 0F
│ Mnemonic:   cmp.w
│ Op String:  ip, #0x28
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0x28 (40)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001EC
│ Bytes:      06 D1
│ Mnemonic:   bne
│ Op String:  #0x80001fc
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80001FC (134218236)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001EE
│ Bytes:      4F F0 01 0C
│ Mnemonic:   mov.w
│ Op String:  ip, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001F2
│ Bytes:      0C FA 00 FC
│ Mnemonic:   lsl.w
│ Op String:  ip, ip, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001F6
│ Bytes:      C2 F8 14 C0
│ Mnemonic:   str.w
│ Op String:  ip, [r2, #0x14]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=20]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001FA
│ Bytes:      0A E0
│ Mnemonic:   b
│ Op String:  #0x8000212
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000212 (134218258)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080001FC
│ Bytes:      91 F8 03 C0
│ Mnemonic:   ldrb.w
│ Op String:  ip, [r1, #3]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=3]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000200
│ Bytes:      BC F1 48 0F
│ Mnemonic:   cmp.w
│ Op String:  ip, #0x48
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0x48 (72)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000204
│ Bytes:      05 D1
│ Mnemonic:   bne
│ Op String:  #0x8000212
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000212 (134218258)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000206
│ Bytes:      4F F0 01 0C
│ Mnemonic:   mov.w
│ Op String:  ip, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800020A
│ Bytes:      0C FA 00 FC
│ Mnemonic:   lsl.w
│ Op String:  ip, ip, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800020E
│ Bytes:      C2 F8 10 C0
│ Mnemonic:   str.w
│ Op String:  ip, [r2, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000212
│ Bytes:      40 1C
│ Mnemonic:   adds
│ Op String:  r0, r0, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000214
│ Bytes:      08 28
│ Mnemonic:   cmp
│ Op String:  r0, #8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x8 (8)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000216
│ Bytes:      D1 D3
│ Mnemonic:   blo
│ Op String:  #0x80001bc
│ Condition:  ARM_CC_LO
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80001BC (134218172)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000218
│ Bytes:      14 60
│ Mnemonic:   str
│ Op String:  r4, [r2]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800021A
│ Bytes:      B1 F8 00 C0
│ Mnemonic:   ldrh.w
│ Op String:  ip, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800021E
│ Bytes:      BC F1 FF 0F
│ Mnemonic:   cmp.w
│ Op String:  ip, #0xff
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0xFF (255)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000222
│ Bytes:      34 DD
│ Mnemonic:   ble
│ Op String:  #0x800028e
│ Condition:  ARM_CC_LE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800028E (134218382)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000224
│ Bytes:      54 68
│ Mnemonic:   ldr
│ Op String:  r4, [r2, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000226
│ Bytes:      00 20
│ Mnemonic:   movs
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000228
│ Bytes:      2E E0
│ Mnemonic:   b
│ Op String:  #0x8000288
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000288 (134218376)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800022A
│ Bytes:      00 F1 08 0C
│ Mnemonic:   add.w
│ Op String:  ip, r0, #8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x8 (8)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800022E
│ Bytes:      4F F0 01 08
│ Mnemonic:   mov.w
│ Op String:  r8, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r8
│       Shift: None
│   [1] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000232
│ Bytes:      08 FA 0C F3
│ Mnemonic:   lsl.w
│ Op String:  r3, r8, ip
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Register: r8
│       Shift: None
│   [2] Type:  Register: ip
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000236
│ Bytes:      B1 F8 00 C0
│ Mnemonic:   ldrh.w
│ Op String:  ip, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800023A
│ Bytes:      0C EA 03 06
│ Mnemonic:   and.w
│ Op String:  r6, ip, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r6
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800023E
│ Bytes:      9E 42
│ Mnemonic:   cmp
│ Op String:  r6, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r6
│       Shift: None
│   [1] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000240
│ Bytes:      21 D1
│ Mnemonic:   bne
│ Op String:  #0x8000286
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000286 (134218374)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000242
│ Bytes:      83 00
│ Mnemonic:   lsls
│ Op String:  r3, r0, #2
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x2 (2)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000244
│ Bytes:      4F F0 0F 0C
│ Mnemonic:   mov.w
│ Op String:  ip, #0xf
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0xF (15)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000248
│ Bytes:      0C FA 03 F7
│ Mnemonic:   lsl.w
│ Op String:  r7, ip, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r7
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800024C
│ Bytes:      BC 43
│ Mnemonic:   bics
│ Op String:  r4, r7
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r7
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800024E
│ Bytes:      05 FA 03 FC
│ Mnemonic:   lsl.w
│ Op String:  ip, r5, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000252
│ Bytes:      4C EA 04 04
│ Mnemonic:   orr.w
│ Op String:  r4, ip, r4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: ip
│       Shift: None
│   [2] Type:  Register: r4
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000256
│ Bytes:      91 F8 03 C0
│ Mnemonic:   ldrb.w
│ Op String:  ip, [r1, #3]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=3]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800025A
│ Bytes:      BC F1 28 0F
│ Mnemonic:   cmp.w
│ Op String:  ip, #0x28
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0x28 (40)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800025E
│ Bytes:      05 D1
│ Mnemonic:   bne
│ Op String:  #0x800026c
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800026C (134218348)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000260
│ Bytes:      00 F1 08 0C
│ Mnemonic:   add.w
│ Op String:  ip, r0, #8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x8 (8)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000264
│ Bytes:      08 FA 0C F8
│ Mnemonic:   lsl.w
│ Op String:  r8, r8, ip
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r8
│       Shift: None
│   [1] Type:  Register: r8
│       Shift: None
│   [2] Type:  Register: ip
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000268
│ Bytes:      C2 F8 14 80
│ Mnemonic:   str.w
│ Op String:  r8, [r2, #0x14]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r8
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=20]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800026C
│ Bytes:      91 F8 03 C0
│ Mnemonic:   ldrb.w
│ Op String:  ip, [r1, #3]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=3]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000270
│ Bytes:      BC F1 48 0F
│ Mnemonic:   cmp.w
│ Op String:  ip, #0x48
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Immediate: 0x48 (72)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000274
│ Bytes:      07 D1
│ Mnemonic:   bne
│ Op String:  #0x8000286
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000286 (134218374)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000276
│ Bytes:      00 F1 08 0C
│ Mnemonic:   add.w
│ Op String:  ip, r0, #8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: ip
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x8 (8)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800027A
│ Bytes:      4F F0 01 08
│ Mnemonic:   mov.w
│ Op String:  r8, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r8
│       Shift: None
│   [1] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800027E
│ Bytes:      08 FA 0C F8
│ Mnemonic:   lsl.w
│ Op String:  r8, r8, ip
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r8
│       Shift: None
│   [1] Type:  Register: r8
│       Shift: None
│   [2] Type:  Register: ip
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000282
│ Bytes:      C2 F8 10 80
│ Mnemonic:   str.w
│ Op String:  r8, [r2, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r8
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000286
│ Bytes:      40 1C
│ Mnemonic:   adds
│ Op String:  r0, r0, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000288
│ Bytes:      08 28
│ Mnemonic:   cmp
│ Op String:  r0, #8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x8 (8)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800028A
│ Bytes:      CE D3
│ Mnemonic:   blo
│ Op String:  #0x800022a
│ Condition:  ARM_CC_LO
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800022A (134218282)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800028C
│ Bytes:      54 60
│ Mnemonic:   str
│ Op String:  r4, [r2, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800028E
│ Bytes:      BD E8 F0 81
│ Mnemonic:   pop.w
│ Op String:  {r4, r5, r6, r7, r8, pc}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (6):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Register: r6
│       Shift: None
│   [3] Type:  Register: r7
│       Shift: None
│   [4] Type:  Register: r8
│       Shift: None
│   [5] Type:  Register: pc
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000292
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000294
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x8000294
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000294 (134218388)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000296
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000298
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x8000298
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000298 (134218392)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800029A
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800029C
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800029E
│ Bytes:      00 00
│ Mnemonic:   movs
│ Op String:  r0, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002A0
│ Bytes:      29 B1
│ Mnemonic:   cbz
│ Op String:  r1, #0x80002ae
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x80002AE (134218414)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002A2
│ Bytes:      06 4A
│ Mnemonic:   ldr
│ Op String:  r2, [pc, #0x18]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=24]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002A4
│ Bytes:      92 69
│ Mnemonic:   ldr
│ Op String:  r2, [r2, #0x18]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=24]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002A6
│ Bytes:      02 43
│ Mnemonic:   orrs
│ Op String:  r2, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002A8
│ Bytes:      04 4B
│ Mnemonic:   ldr
│ Op String:  r3, [pc, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002AA
│ Bytes:      9A 61
│ Mnemonic:   str
│ Op String:  r2, [r3, #0x18]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Memory [base=r3, index=none, scale=1, disp=24]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002AC
│ Bytes:      04 E0
│ Mnemonic:   b
│ Op String:  #0x80002b8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80002B8 (134218424)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002AE
│ Bytes:      03 4A
│ Mnemonic:   ldr
│ Op String:  r2, [pc, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002B0
│ Bytes:      92 69
│ Mnemonic:   ldr
│ Op String:  r2, [r2, #0x18]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=24]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002B2
│ Bytes:      82 43
│ Mnemonic:   bics
│ Op String:  r2, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002B4
│ Bytes:      01 4B
│ Mnemonic:   ldr
│ Op String:  r3, [pc, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002B6
│ Bytes:      9A 61
│ Mnemonic:   str
│ Op String:  r2, [r3, #0x18]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Memory [base=r3, index=none, scale=1, disp=24]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002B8
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None

════════════════════════════════════════════════════════════════════════════
FUNCTION: <RCC_GetClocksFreq>
════════════════════════════════════════════════════════════════════════════
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002C0
│ Bytes:      30 B5
│ Mnemonic:   push
│ Op String:  {r4, r5, lr}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002C2
│ Bytes:      00 21
│ Mnemonic:   movs
│ Op String:  r1, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002C4
│ Bytes:      00 22
│ Mnemonic:   movs
│ Op String:  r2, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002C6
│ Bytes:      00 24
│ Mnemonic:   movs
│ Op String:  r4, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002C8
│ Bytes:      00 23
│ Mnemonic:   movs
│ Op String:  r3, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002CA
│ Bytes:      2D 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0xb4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=180]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002CC
│ Bytes:      6D 68
│ Mnemonic:   ldr
│ Op String:  r5, [r5, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002CE
│ Bytes:      05 F0 0C 01
│ Mnemonic:   and
│ Op String:  r1, r5, #0xc
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Immediate: 0xC (12)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002D2
│ Bytes:      21 B1
│ Mnemonic:   cbz
│ Op String:  r1, #0x80002de
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x80002DE (134218462)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002D4
│ Bytes:      04 29
│ Mnemonic:   cmp
│ Op String:  r1, #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x4 (4)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002D6
│ Bytes:      05 D0
│ Mnemonic:   beq
│ Op String:  #0x80002e4
│ Condition:  ARM_CC_EQ
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80002E4 (134218468)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002D8
│ Bytes:      08 29
│ Mnemonic:   cmp
│ Op String:  r1, #8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x8 (8)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002DA
│ Bytes:      23 D1
│ Mnemonic:   bne
│ Op String:  #0x8000324
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000324 (134218532)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002DC
│ Bytes:      05 E0
│ Mnemonic:   b
│ Op String:  #0x80002ea
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80002EA (134218474)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002DE
│ Bytes:      29 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0xa4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=164]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002E0
│ Bytes:      05 60
│ Mnemonic:   str
│ Op String:  r5, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002E2
│ Bytes:      22 E0
│ Mnemonic:   b
│ Op String:  #0x800032a
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800032A (134218538)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002E4
│ Bytes:      27 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x9c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=156]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002E6
│ Bytes:      05 60
│ Mnemonic:   str
│ Op String:  r5, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002E8
│ Bytes:      1F E0
│ Mnemonic:   b
│ Op String:  #0x800032a
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800032A (134218538)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002EA
│ Bytes:      25 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x94]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=148]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002EC
│ Bytes:      6D 68
│ Mnemonic:   ldr
│ Op String:  r5, [r5, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002EE
│ Bytes:      05 F4 70 12
│ Mnemonic:   and
│ Op String:  r2, r5, #0x3c0000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Immediate: 0x3C0000 (3932160)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002F2
│ Bytes:      23 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x8c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=140]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002F4
│ Bytes:      6D 68
│ Mnemonic:   ldr
│ Op String:  r5, [r5, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002F6
│ Bytes:      05 F4 80 34
│ Mnemonic:   and
│ Op String:  r4, r5, #0x10000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Immediate: 0x10000 (65536)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002FA
│ Bytes:      02 25
│ Mnemonic:   movs
│ Op String:  r5, #2
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Immediate: 0x2 (2)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080002FC
│ Bytes:      05 EB 92 42
│ Mnemonic:   add.w
│ Op String:  r2, r5, r2, lsr #18
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Register: r2
│       Shift: LSR #18
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000300
│ Bytes:      1C B9
│ Mnemonic:   cbnz
│ Op String:  r4, #0x800030a
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Immediate: 0x800030A (134218506)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000302
│ Bytes:      21 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x84]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=132]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000304
│ Bytes:      55 43
│ Mnemonic:   muls
│ Op String:  r5, r2, r5
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Register: r2
│       Shift: None
│   [2] Type:  Register: r5
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000306
│ Bytes:      05 60
│ Mnemonic:   str
│ Op String:  r5, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000308
│ Bytes:      0B E0
│ Mnemonic:   b
│ Op String:  #0x8000322
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000322 (134218530)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800030A
│ Bytes:      1D 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x74]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=116]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800030C
│ Bytes:      6D 68
│ Mnemonic:   ldr
│ Op String:  r5, [r5, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800030E
│ Bytes:      05 F4 00 35
│ Mnemonic:   and
│ Op String:  r5, r5, #0x20000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Immediate: 0x20000 (131072)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000312
│ Bytes:      1D B1
│ Mnemonic:   cbz
│ Op String:  r5, #0x800031c
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Immediate: 0x800031C (134218524)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000314
│ Bytes:      1C 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x70]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=112]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000316
│ Bytes:      55 43
│ Mnemonic:   muls
│ Op String:  r5, r2, r5
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Register: r2
│       Shift: None
│   [2] Type:  Register: r5
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000318
│ Bytes:      05 60
│ Mnemonic:   str
│ Op String:  r5, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800031A
│ Bytes:      02 E0
│ Mnemonic:   b
│ Op String:  #0x8000322
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000322 (134218530)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800031C
│ Bytes:      19 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x64]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=100]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800031E
│ Bytes:      55 43
│ Mnemonic:   muls
│ Op String:  r5, r2, r5
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Register: r2
│       Shift: None
│   [2] Type:  Register: r5
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000320
│ Bytes:      05 60
│ Mnemonic:   str
│ Op String:  r5, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000322
│ Bytes:      02 E0
│ Mnemonic:   b
│ Op String:  #0x800032a
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800032A (134218538)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000324
│ Bytes:      17 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x5c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=92]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000326
│ Bytes:      05 60
│ Mnemonic:   str
│ Op String:  r5, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000328
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800032A
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800032C
│ Bytes:      14 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x50]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=80]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800032E
│ Bytes:      6D 68
│ Mnemonic:   ldr
│ Op String:  r5, [r5, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000330
│ Bytes:      05 F0 F0 01
│ Mnemonic:   and
│ Op String:  r1, r5, #0xf0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Immediate: 0xF0 (240)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000334
│ Bytes:      09 09
│ Mnemonic:   lsrs
│ Op String:  r1, r1, #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
│   [2] Type:  Immediate: 0x4 (4)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000336
│ Bytes:      15 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x54]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=84]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000338
│ Bytes:      6B 5C
│ Mnemonic:   ldrb
│ Op String:  r3, [r5, r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Memory [base=r5, index=r1, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800033A
│ Bytes:      05 68
│ Mnemonic:   ldr
│ Op String:  r5, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800033C
│ Bytes:      DD 40
│ Mnemonic:   lsrs
│ Op String:  r5, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800033E
│ Bytes:      45 60
│ Mnemonic:   str
│ Op String:  r5, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000340
│ Bytes:      0F 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x3c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=60]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000342
│ Bytes:      6D 68
│ Mnemonic:   ldr
│ Op String:  r5, [r5, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000344
│ Bytes:      05 F4 E0 61
│ Mnemonic:   and
│ Op String:  r1, r5, #0x700
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Immediate: 0x700 (1792)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000348
│ Bytes:      09 0A
│ Mnemonic:   lsrs
│ Op String:  r1, r1, #8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
│   [2] Type:  Immediate: 0x8 (8)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800034A
│ Bytes:      10 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x40]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=64]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800034C
│ Bytes:      6B 5C
│ Mnemonic:   ldrb
│ Op String:  r3, [r5, r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Memory [base=r5, index=r1, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800034E
│ Bytes:      45 68
│ Mnemonic:   ldr
│ Op String:  r5, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000350
│ Bytes:      DD 40
│ Mnemonic:   lsrs
│ Op String:  r5, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000352
│ Bytes:      85 60
│ Mnemonic:   str
│ Op String:  r5, [r0, #8]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=8]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000354
│ Bytes:      0A 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x28]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=40]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000356
│ Bytes:      6D 68
│ Mnemonic:   ldr
│ Op String:  r5, [r5, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000358
│ Bytes:      05 F4 60 51
│ Mnemonic:   and
│ Op String:  r1, r5, #0x3800
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Immediate: 0x3800 (14336)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800035C
│ Bytes:      C9 0A
│ Mnemonic:   lsrs
│ Op String:  r1, r1, #0xb
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
│   [2] Type:  Immediate: 0xB (11)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800035E
│ Bytes:      0B 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x2c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=44]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000360
│ Bytes:      6B 5C
│ Mnemonic:   ldrb
│ Op String:  r3, [r5, r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Memory [base=r5, index=r1, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000362
│ Bytes:      45 68
│ Mnemonic:   ldr
│ Op String:  r5, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000364
│ Bytes:      DD 40
│ Mnemonic:   lsrs
│ Op String:  r5, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000366
│ Bytes:      C5 60
│ Mnemonic:   str
│ Op String:  r5, [r0, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000368
│ Bytes:      05 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x14]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=20]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800036A
│ Bytes:      6D 68
│ Mnemonic:   ldr
│ Op String:  r5, [r5, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800036C
│ Bytes:      05 F4 40 41
│ Mnemonic:   and
│ Op String:  r1, r5, #0xc000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Immediate: 0xC000 (49152)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000370
│ Bytes:      89 0B
│ Mnemonic:   lsrs
│ Op String:  r1, r1, #0xe
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
│   [2] Type:  Immediate: 0xE (14)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000372
│ Bytes:      07 4D
│ Mnemonic:   ldr
│ Op String:  r5, [pc, #0x1c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=28]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000374
│ Bytes:      6B 5C
│ Mnemonic:   ldrb
│ Op String:  r3, [r5, r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Memory [base=r5, index=r1, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000376
│ Bytes:      C5 68
│ Mnemonic:   ldr
│ Op String:  r5, [r0, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000378
│ Bytes:      B5 FB F3 F5
│ Mnemonic:   udiv
│ Op String:  r5, r5, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800037C
│ Bytes:      05 61
│ Mnemonic:   str
│ Op String:  r5, [r0, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800037E
│ Bytes:      30 BD
│ Mnemonic:   pop
│ Op String:  {r4, r5, pc}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Register: pc
│       Shift: None

════════════════════════════════════════════════════════════════════════════
FUNCTION: <SVC_Handler>
════════════════════════════════════════════════════════════════════════════
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000394
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000396
│ Bytes:      00 00
│ Mnemonic:   movs
│ Op String:  r0, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000398
│ Bytes:      00 B5
│ Mnemonic:   push
│ Op String:  {lr}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800039A
│ Bytes:      85 B0
│ Mnemonic:   sub
│ Op String:  sp, #0x14
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: sp
│       Shift: None
│   [1] Type:  Immediate: 0x14 (20)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800039C
│ Bytes:      01 21
│ Mnemonic:   movs
│ Op String:  r1, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800039E
│ Bytes:      88 03
│ Mnemonic:   lsls
│ Op String:  r0, r1, #0xe
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
│   [2] Type:  Immediate: 0xE (14)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003A0
│ Bytes:      FF F7 7E FF
│ Mnemonic:   bl
│ Op String:  #0x80002a0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80002A0 (134218400)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003A4
│ Bytes:      01 21
│ Mnemonic:   movs
│ Op String:  r1, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003A6
│ Bytes:      04 20
│ Mnemonic:   movs
│ Op String:  r0, #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x4 (4)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003A8
│ Bytes:      FF F7 7A FF
│ Mnemonic:   bl
│ Op String:  #0x80002a0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80002A0 (134218400)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003AC
│ Bytes:      18 20
│ Mnemonic:   movs
│ Op String:  r0, #0x18
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x18 (24)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003AE
│ Bytes:      8D F8 13 00
│ Mnemonic:   strb.w
│ Op String:  r0, [sp, #0x13]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=19]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003B2
│ Bytes:      4F F4 00 70
│ Mnemonic:   mov.w
│ Op String:  r0, #0x200
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x200 (512)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003B6
│ Bytes:      AD F8 10 00
│ Mnemonic:   strh.w
│ Op String:  r0, [sp, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003BA
│ Bytes:      03 20
│ Mnemonic:   movs
│ Op String:  r0, #3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x3 (3)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003BC
│ Bytes:      8D F8 12 00
│ Mnemonic:   strb.w
│ Op String:  r0, [sp, #0x12]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=18]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003C0
│ Bytes:      04 A9
│ Mnemonic:   add
│ Op String:  r1, sp, #0x10
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: sp
│       Shift: None
│   [2] Type:  Immediate: 0x10 (16)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003C2
│ Bytes:      0E 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0x38]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=56]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003C4
│ Bytes:      FF F7 DA FE
│ Mnemonic:   bl
│ Op String:  #0x800017c
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800017C (134218108)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003C8
│ Bytes:      4F F4 16 50
│ Mnemonic:   mov.w
│ Op String:  r0, #0x2580
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x2580 (9600)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003CC
│ Bytes:      00 90
│ Mnemonic:   str
│ Op String:  r0, [sp]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003CE
│ Bytes:      00 20
│ Mnemonic:   movs
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003D0
│ Bytes:      AD F8 0C 00
│ Mnemonic:   strh.w
│ Op String:  r0, [sp, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003D4
│ Bytes:      08 20
│ Mnemonic:   movs
│ Op String:  r0, #8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x8 (8)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003D6
│ Bytes:      AD F8 0A 00
│ Mnemonic:   strh.w
│ Op String:  r0, [sp, #0xa]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=10]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003DA
│ Bytes:      00 20
│ Mnemonic:   movs
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003DC
│ Bytes:      AD F8 08 00
│ Mnemonic:   strh.w
│ Op String:  r0, [sp, #8]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=8]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003E0
│ Bytes:      AD F8 06 00
│ Mnemonic:   strh.w
│ Op String:  r0, [sp, #6]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=6]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003E4
│ Bytes:      AD F8 04 00
│ Mnemonic:   strh.w
│ Op String:  r0, [sp, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003E8
│ Bytes:      69 46
│ Mnemonic:   mov
│ Op String:  r1, sp
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: sp
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003EA
│ Bytes:      05 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0x14]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=20]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003EC
│ Bytes:      00 F0 DA F8
│ Mnemonic:   bl
│ Op String:  #0x80005a4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80005A4 (134219172)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003F0
│ Bytes:      01 21
│ Mnemonic:   movs
│ Op String:  r1, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003F2
│ Bytes:      03 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003F4
│ Bytes:      00 F0 BC F8
│ Mnemonic:   bl
│ Op String:  #0x8000570
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000570 (134219120)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003F8
│ Bytes:      05 B0
│ Mnemonic:   add
│ Op String:  sp, #0x14
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: sp
│       Shift: None
│   [1] Type:  Immediate: 0x14 (20)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080003FA
│ Bytes:      00 BD
│ Mnemonic:   pop
│ Op String:  {pc}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: pc
│       Shift: None

════════════════════════════════════════════════════════════════════════════
FUNCTION: <Serial_SendByte>
════════════════════════════════════════════════════════════════════════════
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000404
│ Bytes:      10 B5
│ Mnemonic:   push
│ Op String:  {r4, lr}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000406
│ Bytes:      04 46
│ Mnemonic:   mov
│ Op String:  r4, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000408
│ Bytes:      21 46
│ Mnemonic:   mov
│ Op String:  r1, r4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r4
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800040A
│ Bytes:      05 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0x14]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=20]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800040C
│ Bytes:      00 F0 36 F9
│ Mnemonic:   bl
│ Op String:  #0x800067c
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800067C (134219388)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000410
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000412
│ Bytes:      80 21
│ Mnemonic:   movs
│ Op String:  r1, #0x80
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x80 (128)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000414
│ Bytes:      02 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #8]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=8]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000416
│ Bytes:      00 F0 B7 F8
│ Mnemonic:   bl
│ Op String:  #0x8000588
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000588 (134219144)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800041A
│ Bytes:      00 28
│ Mnemonic:   cmp
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800041C
│ Bytes:      F9 D0
│ Mnemonic:   beq
│ Op String:  #0x8000412
│ Condition:  ARM_CC_EQ
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000412 (134218770)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800041E
│ Bytes:      10 BD
│ Mnemonic:   pop
│ Op String:  {r4, pc}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: pc
│       Shift: None

════════════════════════════════════════════════════════════════════════════
FUNCTION: <SetSysClock>
════════════════════════════════════════════════════════════════════════════
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000424
│ Bytes:      10 B5
│ Mnemonic:   push
│ Op String:  {r4, lr}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000426
│ Bytes:      00 F0 01 F8
│ Mnemonic:   bl
│ Op String:  #0x800042c
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800042C (134218796)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800042A
│ Bytes:      10 BD
│ Mnemonic:   pop
│ Op String:  {r4, pc}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: pc
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800042C
│ Bytes:      0C B5
│ Mnemonic:   push
│ Op String:  {r2, r3, lr}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r3
│       Shift: None
│   [2] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800042E
│ Bytes:      00 20
│ Mnemonic:   movs
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000430
│ Bytes:      01 90
│ Mnemonic:   str
│ Op String:  r0, [sp, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000432
│ Bytes:      00 90
│ Mnemonic:   str
│ Op String:  r0, [sp]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000434
│ Bytes:      33 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0xcc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=204]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000436
│ Bytes:      00 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000438
│ Bytes:      40 F4 80 30
│ Mnemonic:   orr
│ Op String:  r0, r0, #0x10000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x10000 (65536)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800043C
│ Bytes:      31 49
│ Mnemonic:   ldr
│ Op String:  r1, [pc, #0xc4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=196]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800043E
│ Bytes:      08 60
│ Mnemonic:   str
│ Op String:  r0, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000440
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000442
│ Bytes:      30 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0xc0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=192]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000444
│ Bytes:      00 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000446
│ Bytes:      00 F4 00 30
│ Mnemonic:   and
│ Op String:  r0, r0, #0x20000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x20000 (131072)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800044A
│ Bytes:      00 90
│ Mnemonic:   str
│ Op String:  r0, [sp]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800044C
│ Bytes:      01 98
│ Mnemonic:   ldr
│ Op String:  r0, [sp, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800044E
│ Bytes:      40 1C
│ Mnemonic:   adds
│ Op String:  r0, r0, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000450
│ Bytes:      01 90
│ Mnemonic:   str
│ Op String:  r0, [sp, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000452
│ Bytes:      00 98
│ Mnemonic:   ldr
│ Op String:  r0, [sp]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000454
│ Bytes:      18 B9
│ Mnemonic:   cbnz
│ Op String:  r0, #0x800045e
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x800045E (134218846)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000456
│ Bytes:      01 98
│ Mnemonic:   ldr
│ Op String:  r0, [sp, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000458
│ Bytes:      B0 F5 A0 6F
│ Mnemonic:   cmp.w
│ Op String:  r0, #0x500
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x500 (1280)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800045C
│ Bytes:      F1 D1
│ Mnemonic:   bne
│ Op String:  #0x8000442
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000442 (134218818)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800045E
│ Bytes:      29 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0xa4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=164]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000460
│ Bytes:      00 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000462
│ Bytes:      00 F4 00 30
│ Mnemonic:   and
│ Op String:  r0, r0, #0x20000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x20000 (131072)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000466
│ Bytes:      10 B1
│ Mnemonic:   cbz
│ Op String:  r0, #0x800046e
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x800046E (134218862)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000468
│ Bytes:      01 20
│ Mnemonic:   movs
│ Op String:  r0, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800046A
│ Bytes:      00 90
│ Mnemonic:   str
│ Op String:  r0, [sp]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800046C
│ Bytes:      01 E0
│ Mnemonic:   b
│ Op String:  #0x8000472
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000472 (134218866)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800046E
│ Bytes:      00 20
│ Mnemonic:   movs
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000470
│ Bytes:      00 90
│ Mnemonic:   str
│ Op String:  r0, [sp]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000472
│ Bytes:      00 98
│ Mnemonic:   ldr
│ Op String:  r0, [sp]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000474
│ Bytes:      01 28
│ Mnemonic:   cmp
│ Op String:  r0, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000476
│ Bytes:      43 D1
│ Mnemonic:   bne
│ Op String:  #0x8000500
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000500 (134219008)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000478
│ Bytes:      23 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0x8c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=140]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800047A
│ Bytes:      00 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800047C
│ Bytes:      40 F0 10 00
│ Mnemonic:   orr
│ Op String:  r0, r0, #0x10
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x10 (16)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000480
│ Bytes:      21 49
│ Mnemonic:   ldr
│ Op String:  r1, [pc, #0x84]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=132]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000482
│ Bytes:      08 60
│ Mnemonic:   str
│ Op String:  r0, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000484
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000486
│ Bytes:      00 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000488
│ Bytes:      20 F0 03 00
│ Mnemonic:   bic
│ Op String:  r0, r0, #3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x3 (3)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800048C
│ Bytes:      08 60
│ Mnemonic:   str
│ Op String:  r0, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800048E
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000490
│ Bytes:      00 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000492
│ Bytes:      40 F0 02 00
│ Mnemonic:   orr
│ Op String:  r0, r0, #2
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x2 (2)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000496
│ Bytes:      08 60
│ Mnemonic:   str
│ Op String:  r0, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000498
│ Bytes:      1A 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0x68]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=104]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800049A
│ Bytes:      40 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800049C
│ Bytes:      19 49
│ Mnemonic:   ldr
│ Op String:  r1, [pc, #0x64]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=100]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800049E
│ Bytes:      48 60
│ Mnemonic:   str
│ Op String:  r0, [r1, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004A0
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004A2
│ Bytes:      40 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004A4
│ Bytes:      48 60
│ Mnemonic:   str
│ Op String:  r0, [r1, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004A6
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004A8
│ Bytes:      40 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004AA
│ Bytes:      40 F4 80 60
│ Mnemonic:   orr
│ Op String:  r0, r0, #0x400
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x400 (1024)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004AE
│ Bytes:      48 60
│ Mnemonic:   str
│ Op String:  r0, [r1, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004B0
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004B2
│ Bytes:      40 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004B4
│ Bytes:      20 F4 7C 10
│ Mnemonic:   bic
│ Op String:  r0, r0, #0x3f0000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x3F0000 (4128768)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004B8
│ Bytes:      48 60
│ Mnemonic:   str
│ Op String:  r0, [r1, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004BA
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004BC
│ Bytes:      40 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004BE
│ Bytes:      40 F4 E8 10
│ Mnemonic:   orr
│ Op String:  r0, r0, #0x1d0000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x1D0000 (1900544)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004C2
│ Bytes:      48 60
│ Mnemonic:   str
│ Op String:  r0, [r1, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004C4
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004C6
│ Bytes:      00 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004C8
│ Bytes:      40 F0 80 70
│ Mnemonic:   orr
│ Op String:  r0, r0, #0x1000000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x1000000 (16777216)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004CC
│ Bytes:      08 60
│ Mnemonic:   str
│ Op String:  r0, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004CE
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004D0
│ Bytes:      0C 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0x30]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=48]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004D2
│ Bytes:      00 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004D4
│ Bytes:      00 F0 00 70
│ Mnemonic:   and
│ Op String:  r0, r0, #0x2000000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x2000000 (33554432)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004D8
│ Bytes:      00 28
│ Mnemonic:   cmp
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004DA
│ Bytes:      F9 D0
│ Mnemonic:   beq
│ Op String:  #0x80004d0
│ Condition:  ARM_CC_EQ
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80004D0 (134218960)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004DC
│ Bytes:      09 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0x24]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=36]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004DE
│ Bytes:      40 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004E0
│ Bytes:      20 F0 03 00
│ Mnemonic:   bic
│ Op String:  r0, r0, #3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x3 (3)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004E4
│ Bytes:      07 49
│ Mnemonic:   ldr
│ Op String:  r1, [pc, #0x1c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=28]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004E6
│ Bytes:      48 60
│ Mnemonic:   str
│ Op String:  r0, [r1, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004E8
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004EA
│ Bytes:      40 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004EC
│ Bytes:      40 F0 02 00
│ Mnemonic:   orr
│ Op String:  r0, r0, #2
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x2 (2)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004F0
│ Bytes:      48 60
│ Mnemonic:   str
│ Op String:  r0, [r1, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004F2
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004F4
│ Bytes:      03 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004F6
│ Bytes:      40 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004F8
│ Bytes:      00 F0 0C 00
│ Mnemonic:   and
│ Op String:  r0, r0, #0xc
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0xC (12)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004FC
│ Bytes:      08 28
│ Mnemonic:   cmp
│ Op String:  r0, #8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x8 (8)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080004FE
│ Bytes:      F9 D1
│ Mnemonic:   bne
│ Op String:  #0x80004f4
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80004F4 (134218996)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000500
│ Bytes:      0C BD
│ Mnemonic:   pop
│ Op String:  {r2, r3, pc}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r3
│       Shift: None
│   [2] Type:  Register: pc
│       Shift: None

════════════════════════════════════════════════════════════════════════════
FUNCTION: <SysTick_Handler>
════════════════════════════════════════════════════════════════════════════
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800050C
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800050E
│ Bytes:      00 00
│ Mnemonic:   movs
│ Op String:  r0, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000510
│ Bytes:      10 B5
│ Mnemonic:   push
│ Op String:  {r4, lr}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000512
│ Bytes:      13 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0x4c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=76]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000514
│ Bytes:      00 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000516
│ Bytes:      40 F0 01 00
│ Mnemonic:   orr
│ Op String:  r0, r0, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800051A
│ Bytes:      11 49
│ Mnemonic:   ldr
│ Op String:  r1, [pc, #0x44]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=68]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800051C
│ Bytes:      08 60
│ Mnemonic:   str
│ Op String:  r0, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800051E
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000520
│ Bytes:      40 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000522
│ Bytes:      10 49
│ Mnemonic:   ldr
│ Op String:  r1, [pc, #0x40]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=64]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000524
│ Bytes:      08 40
│ Mnemonic:   ands
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000526
│ Bytes:      0E 49
│ Mnemonic:   ldr
│ Op String:  r1, [pc, #0x38]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=56]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000528
│ Bytes:      48 60
│ Mnemonic:   str
│ Op String:  r0, [r1, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800052A
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800052C
│ Bytes:      00 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800052E
│ Bytes:      0E 49
│ Mnemonic:   ldr
│ Op String:  r1, [pc, #0x38]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=56]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000530
│ Bytes:      08 40
│ Mnemonic:   ands
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000532
│ Bytes:      0B 49
│ Mnemonic:   ldr
│ Op String:  r1, [pc, #0x2c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=44]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000534
│ Bytes:      08 60
│ Mnemonic:   str
│ Op String:  r0, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000536
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000538
│ Bytes:      00 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800053A
│ Bytes:      20 F4 80 20
│ Mnemonic:   bic
│ Op String:  r0, r0, #0x40000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x40000 (262144)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800053E
│ Bytes:      08 60
│ Mnemonic:   str
│ Op String:  r0, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000540
│ Bytes:      08 46
│ Mnemonic:   mov
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000542
│ Bytes:      40 68
│ Mnemonic:   ldr
│ Op String:  r0, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000544
│ Bytes:      20 F4 FE 00
│ Mnemonic:   bic
│ Op String:  r0, r0, #0x7f0000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x7F0000 (8323072)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000548
│ Bytes:      48 60
│ Mnemonic:   str
│ Op String:  r0, [r1, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800054A
│ Bytes:      4F F4 1F 00
│ Mnemonic:   mov.w
│ Op String:  r0, #0x9f0000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x9F0000 (10420224)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800054E
│ Bytes:      88 60
│ Mnemonic:   str
│ Op String:  r0, [r1, #8]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=8]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000550
│ Bytes:      FF F7 68 FF
│ Mnemonic:   bl
│ Op String:  #0x8000424
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000424 (134218788)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000554
│ Bytes:      4F F0 00 60
│ Mnemonic:   mov.w
│ Op String:  r0, #0x8000000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x8000000 (134217728)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000558
│ Bytes:      04 49
│ Mnemonic:   ldr
│ Op String:  r1, [pc, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800055A
│ Bytes:      08 60
│ Mnemonic:   str
│ Op String:  r0, [r1]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r1, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800055C
│ Bytes:      10 BD
│ Mnemonic:   pop
│ Op String:  {r4, pc}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: pc
│       Shift: None

════════════════════════════════════════════════════════════════════════════
FUNCTION: <USART_Cmd>
════════════════════════════════════════════════════════════════════════════
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000570
│ Bytes:      21 B1
│ Mnemonic:   cbz
│ Op String:  r1, #0x800057c
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x800057C (134219132)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000572
│ Bytes:      82 89
│ Mnemonic:   ldrh
│ Op String:  r2, [r0, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000574
│ Bytes:      42 F4 00 52
│ Mnemonic:   orr
│ Op String:  r2, r2, #0x2000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r2
│       Shift: None
│   [2] Type:  Immediate: 0x2000 (8192)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000578
│ Bytes:      82 81
│ Mnemonic:   strh
│ Op String:  r2, [r0, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800057A
│ Bytes:      04 E0
│ Mnemonic:   b
│ Op String:  #0x8000586
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000586 (134219142)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800057C
│ Bytes:      82 89
│ Mnemonic:   ldrh
│ Op String:  r2, [r0, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800057E
│ Bytes:      4D F6 FF 73
│ Mnemonic:   movw
│ Op String:  r3, #0xdfff
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Immediate: 0xDFFF (57343)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000582
│ Bytes:      1A 40
│ Mnemonic:   ands
│ Op String:  r2, r3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000584
│ Bytes:      82 81
│ Mnemonic:   strh
│ Op String:  r2, [r0, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000586
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000588
│ Bytes:      02 46
│ Mnemonic:   mov
│ Op String:  r2, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800058A
│ Bytes:      00 20
│ Mnemonic:   movs
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800058C
│ Bytes:      B1 F5 00 7F
│ Mnemonic:   cmp.w
│ Op String:  r1, #0x200
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x200 (512)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000590
│ Bytes:      00 D1
│ Mnemonic:   bne
│ Op String:  #0x8000594
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000594 (134219156)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000592
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000594
│ Bytes:      13 88
│ Mnemonic:   ldrh
│ Op String:  r3, [r2]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Memory [base=r2, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000596
│ Bytes:      0B 40
│ Mnemonic:   ands
│ Op String:  r3, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000598
│ Bytes:      0B B1
│ Mnemonic:   cbz
│ Op String:  r3, #0x800059e
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r3
│       Shift: None
│   [1] Type:  Immediate: 0x800059E (134219166)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800059A
│ Bytes:      01 20
│ Mnemonic:   movs
│ Op String:  r0, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800059C
│ Bytes:      00 E0
│ Mnemonic:   b
│ Op String:  #0x80005a0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80005A0 (134219168)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800059E
│ Bytes:      00 20
│ Mnemonic:   movs
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005A0
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005A2
│ Bytes:      00 00
│ Mnemonic:   movs
│ Op String:  r0, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005A4
│ Bytes:      2D E9 F0 47
│ Mnemonic:   push.w
│ Op String:  {r4, r5, r6, r7, r8, sb, sl, lr}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (8):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Register: r6
│       Shift: None
│   [3] Type:  Register: r7
│       Shift: None
│   [4] Type:  Register: r8
│       Shift: None
│   [5] Type:  Register: sb
│       Shift: None
│   [6] Type:  Register: sl
│       Shift: None
│   [7] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005A8
│ Bytes:      86 B0
│ Mnemonic:   sub
│ Op String:  sp, #0x18
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: sp
│       Shift: None
│   [1] Type:  Immediate: 0x18 (24)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005AA
│ Bytes:      05 46
│ Mnemonic:   mov
│ Op String:  r5, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r5
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005AC
│ Bytes:      0E 46
│ Mnemonic:   mov
│ Op String:  r6, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r6
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005AE
│ Bytes:      00 24
│ Mnemonic:   movs
│ Op String:  r4, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005B0
│ Bytes:      A2 46
│ Mnemonic:   mov
│ Op String:  sl, r4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: sl
│       Shift: None
│   [1] Type:  Register: r4
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005B2
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005B4
│ Bytes:      A1 46
│ Mnemonic:   mov
│ Op String:  sb, r4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: sb
│       Shift: None
│   [1] Type:  Register: r4
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005B6
│ Bytes:      00 27
│ Mnemonic:   movs
│ Op String:  r7, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r7
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005B8
│ Bytes:      B0 89
│ Mnemonic:   ldrh
│ Op String:  r0, [r6, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r6, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005BA
│ Bytes:      00 B1
│ Mnemonic:   cbz
│ Op String:  r0, #0x80005be
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x80005BE (134219198)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005BC
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005BE
│ Bytes:      2F 46
│ Mnemonic:   mov
│ Op String:  r7, r5
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r7
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005C0
│ Bytes:      2C 8A
│ Mnemonic:   ldrh
│ Op String:  r4, [r5, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005C2
│ Bytes:      4C F6 FF 70
│ Mnemonic:   movw
│ Op String:  r0, #0xcfff
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0xCFFF (53247)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005C6
│ Bytes:      04 40
│ Mnemonic:   ands
│ Op String:  r4, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005C8
│ Bytes:      F0 88
│ Mnemonic:   ldrh
│ Op String:  r0, [r6, #6]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r6, index=none, scale=1, disp=6]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005CA
│ Bytes:      04 43
│ Mnemonic:   orrs
│ Op String:  r4, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005CC
│ Bytes:      2C 82
│ Mnemonic:   strh
│ Op String:  r4, [r5, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005CE
│ Bytes:      AC 89
│ Mnemonic:   ldrh
│ Op String:  r4, [r5, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005D0
│ Bytes:      4E F6 F3 10
│ Mnemonic:   movw
│ Op String:  r0, #0xe9f3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0xE9F3 (59891)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005D4
│ Bytes:      04 40
│ Mnemonic:   ands
│ Op String:  r4, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005D6
│ Bytes:      B0 88
│ Mnemonic:   ldrh
│ Op String:  r0, [r6, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r6, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005D8
│ Bytes:      31 89
│ Mnemonic:   ldrh
│ Op String:  r1, [r6, #8]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=r6, index=none, scale=1, disp=8]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005DA
│ Bytes:      08 43
│ Mnemonic:   orrs
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005DC
│ Bytes:      71 89
│ Mnemonic:   ldrh
│ Op String:  r1, [r6, #0xa]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=r6, index=none, scale=1, disp=10]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005DE
│ Bytes:      08 43
│ Mnemonic:   orrs
│ Op String:  r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005E0
│ Bytes:      04 43
│ Mnemonic:   orrs
│ Op String:  r4, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005E2
│ Bytes:      AC 81
│ Mnemonic:   strh
│ Op String:  r4, [r5, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005E4
│ Bytes:      AC 8A
│ Mnemonic:   ldrh
│ Op String:  r4, [r5, #0x14]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=20]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005E6
│ Bytes:      4F F6 FF 40
│ Mnemonic:   movw
│ Op String:  r0, #0xfcff
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0xFCFF (64767)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005EA
│ Bytes:      04 40
│ Mnemonic:   ands
│ Op String:  r4, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005EC
│ Bytes:      B0 89
│ Mnemonic:   ldrh
│ Op String:  r0, [r6, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r6, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005EE
│ Bytes:      04 43
│ Mnemonic:   orrs
│ Op String:  r4, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005F0
│ Bytes:      AC 82
│ Mnemonic:   strh
│ Op String:  r4, [r5, #0x14]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=20]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005F2
│ Bytes:      01 A8
│ Mnemonic:   add
│ Op String:  r0, sp, #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: sp
│       Shift: None
│   [2] Type:  Immediate: 0x4 (4)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005F4
│ Bytes:      FF F7 64 FE
│ Mnemonic:   bl
│ Op String:  #0x80002c0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80002C0 (134218432)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005F8
│ Bytes:      1F 48
│ Mnemonic:   ldr
│ Op String:  r0, [pc, #0x7c]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=pc, index=none, scale=1, disp=124]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005FA
│ Bytes:      87 42
│ Mnemonic:   cmp
│ Op String:  r7, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r7
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005FC
│ Bytes:      02 D1
│ Mnemonic:   bne
│ Op String:  #0x8000604
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000604 (134219268)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080005FE
│ Bytes:      DD F8 10 A0
│ Mnemonic:   ldr.w
│ Op String:  sl, [sp, #0x10]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: sl
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=16]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000602
│ Bytes:      01 E0
│ Mnemonic:   b
│ Op String:  #0x8000608
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000608 (134219272)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000604
│ Bytes:      DD F8 0C A0
│ Mnemonic:   ldr.w
│ Op String:  sl, [sp, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: sl
│       Shift: None
│   [1] Type:  Memory [base=sp, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000608
│ Bytes:      A8 89
│ Mnemonic:   ldrh
│ Op String:  r0, [r5, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800060A
│ Bytes:      00 F4 00 40
│ Mnemonic:   and
│ Op String:  r0, r0, #0x8000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x8000 (32768)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800060E
│ Bytes:      40 B1
│ Mnemonic:   cbz
│ Op String:  r0, #0x8000622
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x8000622 (134219298)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000610
│ Bytes:      0A EB CA 00
│ Mnemonic:   add.w
│ Op String:  r0, sl, sl, lsl #3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: sl
│       Shift: None
│   [2] Type:  Register: sl
│       Shift: LSL #3
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000614
│ Bytes:      00 EB 0A 10
│ Mnemonic:   add.w
│ Op String:  r0, r0, sl, lsl #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Register: sl
│       Shift: LSL #4
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000618
│ Bytes:      31 68
│ Mnemonic:   ldr
│ Op String:  r1, [r6]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=r6, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800061A
│ Bytes:      49 00
│ Mnemonic:   lsls
│ Op String:  r1, r1, #1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
│   [2] Type:  Immediate: 0x1 (1)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800061C
│ Bytes:      B0 FB F1 F8
│ Mnemonic:   udiv
│ Op String:  r8, r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r8
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000620
│ Bytes:      07 E0
│ Mnemonic:   b
│ Op String:  #0x8000632
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000632 (134219314)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000622
│ Bytes:      0A EB CA 00
│ Mnemonic:   add.w
│ Op String:  r0, sl, sl, lsl #3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: sl
│       Shift: None
│   [2] Type:  Register: sl
│       Shift: LSL #3
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000626
│ Bytes:      00 EB 0A 10
│ Mnemonic:   add.w
│ Op String:  r0, r0, sl, lsl #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Register: sl
│       Shift: LSL #4
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800062A
│ Bytes:      31 68
│ Mnemonic:   ldr
│ Op String:  r1, [r6]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Memory [base=r6, index=none, scale=1, disp=0]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800062C
│ Bytes:      89 00
│ Mnemonic:   lsls
│ Op String:  r1, r1, #2
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
│   [2] Type:  Immediate: 0x2 (2)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800062E
│ Bytes:      B0 FB F1 F8
│ Mnemonic:   udiv
│ Op String:  r8, r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r8
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000632
│ Bytes:      64 20
│ Mnemonic:   movs
│ Op String:  r0, #0x64
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x64 (100)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000634
│ Bytes:      B8 FB F0 F0
│ Mnemonic:   udiv
│ Op String:  r0, r8, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r8
│       Shift: None
│   [2] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000638
│ Bytes:      04 01
│ Mnemonic:   lsls
│ Op String:  r4, r0, #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x4 (4)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800063A
│ Bytes:      20 09
│ Mnemonic:   lsrs
│ Op String:  r0, r4, #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r4
│       Shift: None
│   [2] Type:  Immediate: 0x4 (4)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800063C
│ Bytes:      64 21
│ Mnemonic:   movs
│ Op String:  r1, #0x64
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x64 (100)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800063E
│ Bytes:      01 FB 10 89
│ Mnemonic:   mls
│ Op String:  sb, r1, r0, r8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (4):
│   [0] Type:  Register: sb
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
│   [2] Type:  Register: r0
│       Shift: None
│   [3] Type:  Register: r8
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000642
│ Bytes:      A8 89
│ Mnemonic:   ldrh
│ Op String:  r0, [r5, #0xc]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=12]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000644
│ Bytes:      00 F4 00 40
│ Mnemonic:   and
│ Op String:  r0, r0, #0x8000
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x8000 (32768)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000648
│ Bytes:      40 B1
│ Mnemonic:   cbz
│ Op String:  r0, #0x800065c
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x800065C (134219356)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800064A
│ Bytes:      32 20
│ Mnemonic:   movs
│ Op String:  r0, #0x32
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x32 (50)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800064C
│ Bytes:      00 EB C9 00
│ Mnemonic:   add.w
│ Op String:  r0, r0, sb, lsl #3
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Register: sb
│       Shift: LSL #3
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000650
│ Bytes:      B0 FB F1 F0
│ Mnemonic:   udiv
│ Op String:  r0, r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000654
│ Bytes:      00 F0 07 00
│ Mnemonic:   and
│ Op String:  r0, r0, #7
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0x7 (7)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000658
│ Bytes:      04 43
│ Mnemonic:   orrs
│ Op String:  r4, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800065A
│ Bytes:      08 E0
│ Mnemonic:   b
│ Op String:  #0x800066e
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800066E (134219374)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800065C
│ Bytes:      32 20
│ Mnemonic:   movs
│ Op String:  r0, #0x32
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x32 (50)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800065E
│ Bytes:      00 EB 09 10
│ Mnemonic:   add.w
│ Op String:  r0, r0, sb, lsl #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Register: sb
│       Shift: LSL #4
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000662
│ Bytes:      64 21
│ Mnemonic:   movs
│ Op String:  r1, #0x64
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Immediate: 0x64 (100)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000664
│ Bytes:      B0 FB F1 F0
│ Mnemonic:   udiv
│ Op String:  r0, r0, r1
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Register: r1
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000668
│ Bytes:      00 F0 0F 00
│ Mnemonic:   and
│ Op String:  r0, r0, #0xf
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
│   [2] Type:  Immediate: 0xF (15)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800066C
│ Bytes:      04 43
│ Mnemonic:   orrs
│ Op String:  r4, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800066E
│ Bytes:      2C 81
│ Mnemonic:   strh
│ Op String:  r4, [r5, #8]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Memory [base=r5, index=none, scale=1, disp=8]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000670
│ Bytes:      06 B0
│ Mnemonic:   add
│ Op String:  sp, #0x18
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: sp
│       Shift: None
│   [1] Type:  Immediate: 0x18 (24)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000672
│ Bytes:      BD E8 F0 87
│ Mnemonic:   pop.w
│ Op String:  {r4, r5, r6, r7, r8, sb, sl, pc}
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (8):
│   [0] Type:  Register: r4
│       Shift: None
│   [1] Type:  Register: r5
│       Shift: None
│   [2] Type:  Register: r6
│       Shift: None
│   [3] Type:  Register: r7
│       Shift: None
│   [4] Type:  Register: r8
│       Shift: None
│   [5] Type:  Register: sb
│       Shift: None
│   [6] Type:  Register: sl
│       Shift: None
│   [7] Type:  Register: pc
│       Shift: None

════════════════════════════════════════════════════════════════════════════
FUNCTION: <USART_SendData>
════════════════════════════════════════════════════════════════════════════
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800067C
│ Bytes:      C1 F3 08 02
│ Mnemonic:   ubfx
│ Op String:  r2, r1, #0, #9
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (4):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r1
│       Shift: None
│   [2] Type:  Immediate: 0x0 (0)
│       Shift: None
│   [3] Type:  Immediate: 0x9 (9)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000680
│ Bytes:      82 80
│ Mnemonic:   strh
│ Op String:  r2, [r0, #4]
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Memory [base=r0, index=none, scale=1, disp=4]
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000682
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000684
│ Bytes:      00 BF
│ Mnemonic:   nop
│ Op String:  
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000686
│ Bytes:      FE E7
│ Mnemonic:   b
│ Op String:  #0x8000686
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000686 (134219398)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000688
│ Bytes:      02 E0
│ Mnemonic:   b
│ Op String:  #0x8000690
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000690 (134219408)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800068A
│ Bytes:      08 C8
│ Mnemonic:   ldm
│ Op String:  r0!, {r3}
│ Condition:  ARM_CC_AL
│ Writeback:  Yes
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800068C
│ Bytes:      12 1F
│ Mnemonic:   subs
│ Op String:  r2, r2, #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r2
│       Shift: None
│   [2] Type:  Immediate: 0x4 (4)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800068E
│ Bytes:      08 C1
│ Mnemonic:   stm
│ Op String:  r1!, {r3}
│ Condition:  ARM_CC_AL
│ Writeback:  Yes
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r3
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000690
│ Bytes:      00 2A
│ Mnemonic:   cmp
│ Op String:  r2, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000692
│ Bytes:      FA D1
│ Mnemonic:   bne
│ Op String:  #0x800068a
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800068A (134219402)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000694
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000696
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x08000698
│ Bytes:      00 20
│ Mnemonic:   movs
│ Op String:  r0, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800069A
│ Bytes:      01 E0
│ Mnemonic:   b
│ Op String:  #0x80006a0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80006A0 (134219424)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800069C
│ Bytes:      01 C1
│ Mnemonic:   stm
│ Op String:  r1!, {r0}
│ Condition:  ARM_CC_AL
│ Writeback:  Yes
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r1
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x0800069E
│ Bytes:      12 1F
│ Mnemonic:   subs
│ Op String:  r2, r2, #4
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (3):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Register: r2
│       Shift: None
│   [2] Type:  Immediate: 0x4 (4)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080006A0
│ Bytes:      00 2A
│ Mnemonic:   cmp
│ Op String:  r2, #0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r2
│       Shift: None
│   [1] Type:  Immediate: 0x0 (0)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080006A2
│ Bytes:      FB D1
│ Mnemonic:   bne
│ Op String:  #0x800069c
│ Condition:  ARM_CC_NE
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800069C (134219420)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080006A4
│ Bytes:      70 47
│ Mnemonic:   bx
│ Op String:  lr
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Register: lr
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080006A6
│ Bytes:      FF F7 77 FE
│ Mnemonic:   bl
│ Op String:  #0x8000398
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000398 (134218648)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080006AA
│ Bytes:      05 E0
│ Mnemonic:   b
│ Op String:  #0x80006b8
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80006B8 (134219448)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080006AC
│ Bytes:      41 20
│ Mnemonic:   movs
│ Op String:  r0, #0x41
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x41 (65)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080006AE
│ Bytes:      FF F7 A9 FE
│ Mnemonic:   bl
│ Op String:  #0x8000404
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x8000404 (134218756)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080006B2
│ Bytes:      64 20
│ Mnemonic:   movs
│ Op String:  r0, #0x64
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Immediate: 0x64 (100)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080006B4
│ Bytes:      FF F7 4B FD
│ Mnemonic:   bl
│ Op String:  #0x800014e
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x800014E (134218062)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080006B8
│ Bytes:      F8 E7
│ Mnemonic:   b
│ Op String:  #0x80006ac
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (1):
│   [0] Type:  Immediate: 0x80006AC (134219436)
│       Shift: None
────────────────────────────────────────────────────────────────────────────
│ Address:    0x080006BA
│ Bytes:      00 00
│ Mnemonic:   movs
│ Op String:  r0, r0
│ Condition:  ARM_CC_AL
│ Writeback:  No
│ Post-Index: No
│
│ Operands (2):
│   [0] Type:  Register: r0
│       Shift: None
│   [1] Type:  Register: r0
│       Shift: None

┌─────────────────────────────────────────────────────────────────────────────┐
│                              DATA SECTION (DCW)                              │
└─────────────────────────────────────────────────────────────────────────────┘

Address        Hex            Value     
----------------------------------------
0x08000000     18 04          DCW  0x0418
0x08000002     00 20          DCW  0x2000
0x08000004     01 01          DCW  0x0101
0x08000006     00 08          DCW  0x0800
0x08000008     9B 02          DCW  0x029B
0x0800000A     00 08          DCW  0x0800
0x0800000C     93 02          DCW  0x0293
0x0800000E     00 08          DCW  0x0800
0x08000010     97 02          DCW  0x0297
0x08000012     00 08          DCW  0x0800
0x08000014     49 01          DCW  0x0149
0x08000016     00 08          DCW  0x0800
0x08000018     85 06          DCW  0x0685
0x0800001A     00 08          DCW  0x0800
0x0800001C     00 00          DCW  0x0000
0x0800001E     00 00          DCW  0x0000
0x08000020     00 00          DCW  0x0000
0x08000022     00 00          DCW  0x0000
0x08000024     00 00          DCW  0x0000
0x08000026     00 00          DCW  0x0000
0x08000028     00 00          DCW  0x0000
0x0800002A     00 00          DCW  0x0000
0x0800002C     95 03          DCW  0x0395
0x0800002E     00 08          DCW  0x0800
0x08000030     4D 01          DCW  0x014D
0x08000032     00 08          DCW  0x0800
0x08000034     00 00          DCW  0x0000
0x08000036     00 00          DCW  0x0000
0x08000038     9D 02          DCW  0x029D
0x0800003A     00 08          DCW  0x0800
0x0800003C     0D 05          DCW  0x050D
0x0800003E     00 08          DCW  0x0800
0x08000040     1B 01          DCW  0x011B
0x08000042     00 08          DCW  0x0800
0x08000044     1B 01          DCW  0x011B
0x08000046     00 08          DCW  0x0800
0x08000048     1B 01          DCW  0x011B
0x0800004A     00 08          DCW  0x0800
0x0800004C     1B 01          DCW  0x011B
0x0800004E     00 08          DCW  0x0800
0x08000050     1B 01          DCW  0x011B
0x08000052     00 08          DCW  0x0800
0x08000054     1B 01          DCW  0x011B
0x08000056     00 08          DCW  0x0800
0x08000058     1B 01          DCW  0x011B
0x0800005A     00 08          DCW  0x0800
0x0800005C     1B 01          DCW  0x011B
0x0800005E     00 08          DCW  0x0800
0x08000060     1B 01          DCW  0x011B
0x08000062     00 08          DCW  0x0800
0x08000064     1B 01          DCW  0x011B
0x08000066     00 08          DCW  0x0800
0x08000068     1B 01          DCW  0x011B
0x0800006A     00 08          DCW  0x0800
0x0800006C     1B 01          DCW  0x011B
0x0800006E     00 08          DCW  0x0800
0x08000070     1B 01          DCW  0x011B
0x08000072     00 08          DCW  0x0800
0x08000074     1B 01          DCW  0x011B
0x08000076     00 08          DCW  0x0800
0x08000078     1B 01          DCW  0x011B
0x0800007A     00 08          DCW  0x0800
0x0800007C     1B 01          DCW  0x011B
0x0800007E     00 08          DCW  0x0800
0x08000080     1B 01          DCW  0x011B
0x08000082     00 08          DCW  0x0800
0x08000084     1B 01          DCW  0x011B
0x08000086     00 08          DCW  0x0800
0x08000088     1B 01          DCW  0x011B
0x0800008A     00 08          DCW  0x0800
0x0800008C     1B 01          DCW  0x011B
0x0800008E     00 08          DCW  0x0800
0x08000090     1B 01          DCW  0x011B
0x08000092     00 08          DCW  0x0800
0x08000094     1B 01          DCW  0x011B
0x08000096     00 08          DCW  0x0800
0x08000098     1B 01          DCW  0x011B
0x0800009A     00 08          DCW  0x0800
0x0800009C     1B 01          DCW  0x011B
0x0800009E     00 08          DCW  0x0800
0x080000A0     1B 01          DCW  0x011B
0x080000A2     00 08          DCW  0x0800
0x080000A4     1B 01          DCW  0x011B
0x080000A6     00 08          DCW  0x0800
0x080000A8     1B 01          DCW  0x011B
0x080000AA     00 08          DCW  0x0800
0x080000AC     1B 01          DCW  0x011B
0x080000AE     00 08          DCW  0x0800
0x080000B0     1B 01          DCW  0x011B
0x080000B2     00 08          DCW  0x0800
0x080000B4     1B 01          DCW  0x011B
0x080000B6     00 08          DCW  0x0800
0x080000B8     1B 01          DCW  0x011B
0x080000BA     00 08          DCW  0x0800
0x080000BC     1B 01          DCW  0x011B
0x080000BE     00 08          DCW  0x0800
0x080000C0     1B 01          DCW  0x011B
0x080000C2     00 08          DCW  0x0800
0x080000C4     1B 01          DCW  0x011B
0x080000C6     00 08          DCW  0x0800
0x080000C8     1B 01          DCW  0x011B
0x080000CA     00 08          DCW  0x0800
0x080000CC     1B 01          DCW  0x011B
0x080000CE     00 08          DCW  0x0800
0x080000D0     1B 01          DCW  0x011B
0x080000D2     00 08          DCW  0x0800
0x080000D4     1B 01          DCW  0x011B
0x080000D6     00 08          DCW  0x0800
0x080000D8     1B 01          DCW  0x011B
0x080000DA     00 08          DCW  0x0800
0x080000DC     1B 01          DCW  0x011B
0x080000DE     00 08          DCW  0x0800
0x080000E0     1B 01          DCW  0x011B
0x080000E2     00 08          DCW  0x0800
0x080000E4     1B 01          DCW  0x011B
0x080000E6     00 08          DCW  0x0800
0x080000E8     1B 01          DCW  0x011B
0x080000EA     00 08          DCW  0x0800
0x080000F8     A7 06          DCW  0x06A7
0x080000FA     00 08          DCW  0x0800
0x080000FC     18 04          DCW  0x0418
0x080000FE     00 20          DCW  0x2000
0x0800011C     11 05          DCW  0x0511
0x0800011E     00 08          DCW  0x0800
0x08000120     ED 00          DCW  0x00ED
0x08000122     00 08          DCW  0x0800
0x08000140     BC 06          DCW  0x06BC
0x08000142     00 08          DCW  0x0800
0x08000144     DC 06          DCW  0x06DC
0x08000146     00 08          DCW  0x0800
0x080002BA     00 00          DCW  0x0000
0x080002BC     00 10          DCW  0x1000
0x080002BE     02 40          DCW  0x4002
0x08000380     00 10          DCW  0x1000
0x08000382     02 40          DCW  0x4002
0x08000384     00 12          DCW  0x1200
0x08000386     7A 00          DCW  0x007A
0x08000388     00 09          DCW  0x0900
0x0800038A     3D 00          DCW  0x003D
0x0800038C     00 00          DCW  0x0000
0x0800038E     00 20          DCW  0x2000
0x08000390     10 00          DCW  0x0010
0x08000392     00 20          DCW  0x2000
0x080003FC     00 08          DCW  0x0800
0x080003FE     01 40          DCW  0x4001
0x08000400     00 38          DCW  0x3800
0x08000402     01 40          DCW  0x4001
0x08000420     00 38          DCW  0x3800
0x08000422     01 40          DCW  0x4001
0x08000502     00 00          DCW  0x0000
0x08000504     00 10          DCW  0x1000
0x08000506     02 40          DCW  0x4002
0x08000508     00 20          DCW  0x2000
0x0800050A     02 40          DCW  0x4002
0x0800055E     00 00          DCW  0x0000
0x08000560     00 10          DCW  0x1000
0x08000562     02 40          DCW  0x4002
0x08000564     00 00          DCW  0x0000
0x08000566     FF F8          DCW  0xF8FF
0x08000568     FF FF          DCW  0xFFFF
0x0800056A     F6 FE          DCW  0xFEF6
0x0800056C     08 ED          DCW  0xED08
0x0800056E     00 E0          DCW  0xE000
0x08000676     00 00          DCW  0x0000
0x08000678     00 38          DCW  0x3800
0x0800067A     01 40          DCW  0x4001
0x080006BC     DC 06          DCW  0x06DC
0x080006BE     00 08          DCW  0x0800
0x080006C0     00 00          DCW  0x0000
0x080006C2     00 20          DCW  0x2000
0x080006C4     14 00          DCW  0x0014
0x080006C6     00 00          DCW  0x0000
0x080006C8     88 06          DCW  0x0688
0x080006CA     00 08          DCW  0x0800
0x080006CC     F0 06          DCW  0x06F0
0x080006CE     00 08          DCW  0x0800
0x080006D0     14 00          DCW  0x0014
0x080006D2     00 20          DCW  0x2000
0x080006D4     04 04          DCW  0x0404
0x080006D6     00 00          DCW  0x0000
0x080006D8     98 06          DCW  0x0698
0x080006DA     00 08          DCW  0x0800

================================================================================
                    总计: 555 条指令, 182 个数据字
================================================================================
