/* Memory layout for QEMU's MPS2-AN385 machine (Cortex-M). Code runs from the
   ZBT SRAM mapped at address 0; data lives in the SRAM at 0x2000_0000. */
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 4M
  RAM   : ORIGIN = 0x20000000, LENGTH = 4M
}
