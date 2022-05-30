## Tcp 7778 Protocol

### Receiving

Everything in little-endian

Sync
```
┌────┬────┬────┬────┐
│ 8  │ 8  │ 8  │ 8  │
├────┼────┼────┼────┤
│0x55│0x55│0x55│0x55│
└────┴────┴────┴────┘
```

Data
```
┌───────┬───┬──────┐
│Address│Len│ Data │
├───────┼───┼──────┤
│  16   │16 │$LEN*8│
└───────┴───┴──────┘
```

If the address is 0x555 expect Sync

If after storing the data at the given Address the address is 0xfffe
probably expect Sync

### Sending

```
<cmd> <args>\n
```
Cmd is the identifier of the control.

if the control interface is `fixed_step` use 'INC' or 'DEC' as argument.