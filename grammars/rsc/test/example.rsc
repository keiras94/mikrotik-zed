# ── Menu commands ──────────────────────────────────────────────
# Basic menu path
/ip route print

# Nested menu path
/ip firewall filter add chain=input action=accept

# Deeply nested menu path
/ip firewall address-list add address=192.168.1.0/24 list=local

# Menu with multiple named params
/interface bridge add name=bridge1 protocol-mode=rstp vlan-filtering=yes mtu=1500

# Menu with unnamed values
/ip dns set servers=8.8.8.8,8.8.4.4

# ── Global commands ────────────────────────────────────────────
:put "hello"
:log info "script started"
:error "aborting"
:return true
:delay 5s

# ── Variable declarations ─────────────────────────────────────
:global myVar 42
:local name "MikroTik"
:local interfaceName "ether1"
:set myVar 100

# ── Variable references ───────────────────────────────────────
:put $myVar
:put $name
/ip route print where gateway=$gateway

# ── Control flow ──────────────────────────────────────────────
:if (yes) do={
  :put "condition true"
}

:if ($var > 10) do={
  :put "large"
} else={
  :put "small"
}

:while ($counter < 100) do={
  :set counter ($counter + 1)
}

:foreach i in=[/ip route find] do={
  :put [/ip route get $i gateway]
}

:do {
  :put "try"
} while=(false)

:for counter from=1 to=10 step=2 do={
  :put $counter
}

# ── Command substitution ──────────────────────────────────────
:local routes [/ip route find]
:local gw [/ip route get 0 gateway]
:put [/ip address get [find where interface=ether1] address]

# ── Subexpressions ────────────────────────────────────────────
:local result (2 + 3)
:local combined ($a . $b)
:if ($x != 0 && $y > 5) do={ }

# ── Arrays ────────────────────────────────────────────────────
:local myList {1;2;3}
:local myMap {key1="value1"; key2="value2"}
:local mixed {name="eth1"; mtu=1500; disabled=no}

# ── Array access ──────────────────────────────────────────────
:put $myList->0
:put $myMap->"key1"

# ── Function calls ────────────────────────────────────────────
$:put "test"
$execute script=backup

# ── Strings ───────────────────────────────────────────────────
:put "simple string"
:put "escape \"quotes\""
:put "back\\slash"
:put "line1\nline2"

# ── Numbers ───────────────────────────────────────────────────
:local int 42
:local hex 0xFF
:local zero 0

# ── Booleans ──────────────────────────────────────────────────
:local flag yes
:local other no
:local tf true
:local ff false

# ── Nil ───────────────────────────────────────────────────────
:local nothing nil

# ── IP addresses and prefixes ─────────────────────────────────
:local ip 192.168.1.1
:local cidr 10.0.0.0/8
:local ipv6 2001:db8::1

# ── Operators ─────────────────────────────────────────────────
:local sum ($a + $b)
:local diff ($a - $b)
:local prod ($a * $b)
:local quot ($a / $b)
:local mod ($a % $b)
:local eq ($a == $b)
:local neq ($a != $b)
:local lt ($a < $b)
:local gt ($a > $b)
:local lte ($a <= $b)
:local gte ($a >= $b)
:local and ($a && $b)
:local or ($a || $b)
:local not (! $a)
:local band ($a & $b)
:local bor ($a | $b)
:local bxor ($a ^ $b)
:local shl ($a << 2)
:local shr ($a >> 2)
:local concat ($a . $b)

# ── Line continuation ─────────────────────────────────────────
:local long ("this is a " . \
  "very long string")

# ── Parent navigation ─────────────────────────────────────────
..
# (used in interactive CLI to go up one menu level)

# ── Comments ──────────────────────────────────────────────────
# This is a comment
# Multi-line
# comment block
