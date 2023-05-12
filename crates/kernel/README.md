# Kernel

The main kernel code of k_os.

## Memory

Memory management is a complicated topic, and it is mostly moved into the
'pager' server.

The kernel guards its own memory. The pager supplies the kernel with necessary
extra pages for running programs etc.

The 'pager' manages the address spaces, the kernel merely adds translations.