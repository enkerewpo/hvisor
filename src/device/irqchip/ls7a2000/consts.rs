// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//      Yulong Han <wheatfox17@icloud.com>
//
// LS7A2000 bridge chip MMIO and PCI layout constants.

use crate::PHY_TO_DMW_UNCACHED;

/// Base address for LS7A2000 MMIO (DMW-uncached).
pub const MMIO_BASE: usize = PHY_TO_DMW_UNCACHED!(0x1fe0_0000);

// UART & legacy interrupt controller around 0x1fe0_0000.
pub const UART0_BASE: usize = MMIO_BASE + 0x01e0;
pub const UART0_SIZE: usize = 0x8;

pub const LIOINTC_MAP_BASE: usize = MMIO_BASE + 0x1400; // 0x1fe0_1400 - 0x1fe0_141f
pub const LIOINTC_MAP_SIZE: usize = 0x20;

pub const ANYSEND_BASE: usize = MMIO_BASE + 0x1158;
pub const ANYSEND_SIZE: usize = 0x10;

pub const EXTIOI_MAP_CORE_BASE: usize = MMIO_BASE + 0x1c00;
pub const EXTIOI_MAP_CORE_SIZE: usize = 0x100;

pub const EXTIOI_SR_BASE: usize = MMIO_BASE + 0x1700; // 0x1fe0_1700 - 0x1fe0_1718
pub const EXTIOI_SR_SIZE: usize = 0x20;

pub const EXTIOI_NODE_SEL_BASE: usize = MMIO_BASE + 0x14a0; // 0x1fe0_14a0 - 0x1fe0_14be
pub const EXTIOI_NODE_SEL_SIZE: usize = 0x20;

// core0: 0x1fe0_1800-0x18ff, core1: 0x1900-0x19ff, core2: 0x1a00-0x1aff, core3: 0x1b00-0x1bff
pub const EXTIOI_SR_CORE_BASE: usize = MMIO_BASE + 0x1800;
pub const EXTIOI_SR_CORE_SIZE: usize = 0x400;

pub const EXTIOI_ENABLE_BASE: usize = MMIO_BASE + 0x1600; // 0x1fe0_1600 - 0x1fe0_1618
pub const EXTIOI_ENABLE_SIZE: usize = 0x20;

pub const EXTIOI_BOUNCE_BASE: usize = MMIO_BASE + 0x1680; // 0x1fe0_1680 - 0x1fe0_1698
pub const EXTIOI_BOUNCE_SIZE: usize = 0x20;

// Chip config / legacy int / extioi register blocks (3A5000 manual).
pub const CHIP_CONFIG_BASE: usize = MMIO_BASE + 0x0;
pub const CHIP_OTHER_FUNCTION_BASE: usize = MMIO_BASE + 0x420;
pub const CHIP_LEGACY_INT_ROUTE_BASE: usize = MMIO_BASE + 0x1400;
pub const CHIP_LEGACY_INT_CTRL_BASE: usize = MMIO_BASE + 0x1420;
pub const CHIP_EXTIOI_DEBUG_SEND_BASE: usize = MMIO_BASE + 0x1140;
pub const CHIP_EXTIOI_NODE_TYPE_BASE: usize = MMIO_BASE + 0x14a0;
pub const CHIP_EXTIOI_ROUTE_BASE: usize = MMIO_BASE + 0x14c2;
pub const CHIP_EXTIOI_ENABLE_BASE: usize = MMIO_BASE + 0x1600;
pub const CHIP_EXTIOI_BOUNCE_BASE: usize = MMIO_BASE + 0x1680;
pub const CHIP_EXTIOI_STATUS_BASE: usize = MMIO_BASE + 0x1700;
pub const CHIP_EXTIOI_CORE0_STATUS_BASE: usize = MMIO_BASE + 0x1800;
pub const CHIP_EXTIOI_CORE1_STATUS_BASE: usize = MMIO_BASE + 0x1900;
pub const CHIP_EXTIOI_CORE2_STATUS_BASE: usize = MMIO_BASE + 0x1a00;
pub const CHIP_EXTIOI_CORE3_STATUS_BASE: usize = MMIO_BASE + 0x1b00;
pub const CHIP_EXTIOI_ROUTE_CORE_BASE: usize = MMIO_BASE + 0x1c00;

// HT and PCI configuration windows.
pub const CHIP_HT_CONFIG_BASE: usize = PHY_TO_DMW_UNCACHED!(0xfd_fb00_0000); // 3A5000 manual p118
pub const CHIP_HT_INT_VECTOR_BASE: usize = CHIP_HT_CONFIG_BASE + 0x80;
pub const CHIP_HT_INT_EN_BASE: usize = CHIP_HT_CONFIG_BASE + 0xa0;

pub const PCI_STANDARD_CONFIG_BASE_ALT: usize = PHY_TO_DMW_UNCACHED!(0x1a00_0000);
pub const PCI_STANDARD_CONFIG_BASE: usize = PHY_TO_DMW_UNCACHED!(0xefd_fe00_0000);
pub const PCI_RESERVED_CONFIG_BASE: usize = PHY_TO_DMW_UNCACHED!(0xefe_0000_0000);

// Loongson PCI IDs (see https://admin.pci-ids.ucw.cz/read/PC/0014).
pub const PCI_VENDOR_ID_LOONGSON: usize = 0x0014;

pub const PCI_DEVICE_ID_HT_BRIDGE: usize = 0x7a00;
pub const PCI_DEVICE_ID_APB: usize = 0x7a02;
pub const PCI_DEVICE_ID_GIGE: usize = 0x7a03;
pub const PCI_DEVICE_ID_OTG_USB: usize = 0x7a04;
pub const PCI_DEVICE_ID_GPU: usize = 0x7a05;
pub const PCI_DEVICE_ID_DC: usize = 0x7a06;
pub const PCI_DEVICE_ID_HDA: usize = 0x7a07;
pub const PCI_DEVICE_ID_SATA: usize = 0x7a08;
pub const PCI_DEVICE_ID_PCI_BRIDGE: usize = 0x7a09;
pub const PCI_DEVICE_ID_SPI: usize = 0x7a0b;
pub const PCI_DEVICE_ID_LPC: usize = 0x7a0c;
pub const PCI_DEVICE_ID_DMA: usize = 0x7a0f;
pub const PCI_DEVICE_ID_HT_BRIDGE2: usize = 0x7a10;
pub const PCI_DEVICE_ID_PCH_GIGE: usize = 0x7a13;
pub const PCI_DEVICE_ID_EHCI_USB: usize = 0x7a14;
pub const PCI_DEVICE_ID_GPU2: usize = 0x7a15;
pub const PCI_DEVICE_ID_SATA3: usize = 0x7a18;
pub const PCI_DEVICE_ID_PCI_BRIDGE2: usize = 0x7a19;
pub const PCI_DEVICE_ID_SPI2: usize = 0x7a1b;
pub const PCI_DEVICE_ID_OHCI_USB: usize = 0x7a24;
pub const PCI_DEVICE_ID_LG100_GPU: usize = 0x7a25;
pub const PCI_DEVICE_ID_I2S: usize = 0x7a27;
pub const PCI_DEVICE_ID_PCI_BRIDGE3: usize = 0x7a29;
pub const PCI_DEVICE_ID_XHCI_USB: usize = 0x7a34;
pub const PCI_DEVICE_ID_DC2: usize = 0x7a36;
pub const PCI_DEVICE_ID_PCIE_X1: usize = 0x7a39;
pub const PCI_DEVICE_ID_PCIE_X4: usize = 0x7a49;
pub const PCI_DEVICE_ID_PCIE_X8: usize = 0x7a59;
pub const PCI_DEVICE_ID_PCIE_X16: usize = 0x7a69;
