export const productUi = {
  colors: {
    primary: 'stone',
    secondary: 'stone',
    neutral: 'stone',
    info: 'stone',
    success: 'emerald',
    warning: 'amber',
    error: 'red',
  } as const,
  button: {
    slots: {
      base: 'min-h-8 justify-center gap-2 rounded-md text-[13px] font-medium',
      leadingIcon: 'size-4',
      trailingIcon: 'size-4',
    },
    variants: { size: { sm: { base: 'px-3 py-2 text-sm gap-2' } } },
    compoundVariants: [
      {
        color: 'neutral' as const,
        variant: 'ghost' as const,
        class:
          'text-toned hover:text-highlighted hover:bg-hover active:bg-selected focus-visible:outline-focus',
      },
    ],
  },
  input: { slots: { root: 'w-full', base: 'min-h-8 text-[13px] rounded-md' } },
  textarea: {
    slots: { root: 'w-full', base: 'text-[13px] rounded-md leading-relaxed' },
  },
  select: {
    slots: {
      base: 'min-h-8 w-full text-[13px] rounded-md',
      content: 'max-w-[calc(100vw-2rem)]',
      item: 'text-[13px] min-h-8',
      value: 'truncate',
    },
  },
  modal: {
    slots: {
      content: 'rounded-lg w-[calc(100vw-2rem)] max-w-2xl max-h-[90dvh]',
      header: 'shrink-0 p-4 sm:p-6',
      body: 'min-h-0 p-4 sm:p-6 flex flex-col gap-4',
      title: 'text-sm font-semibold',
      wrapper: 'min-w-0 pe-8',
      description: 'text-xs leading-relaxed',
      overlay: 'bg-overlay',
    },
    variants: {
      fullscreen: {
        false: { content: 'max-w-2xl rounded-lg shadow-lg ring ring-default' },
      },
      overlay: { true: { overlay: 'bg-overlay' } },
    },
  },
  slideover: {
    slots: {
      content: 'bg-navigation max-w-xs',
      title: 'text-sm',
      overlay: 'bg-overlay',
    },
    variants: {
      side: { left: { content: 'max-w-xs' }, right: { content: 'max-w-xs' } },
    },
  },
  tooltip: { slots: { content: 'text-xs' } },
}

export const tooltipContent = { side: 'right' as const, sideOffset: 8 }
