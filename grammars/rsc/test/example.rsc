# Sample RouterOS script — Phase 1 core functionality
/ip dhcp-client add interface=ether1 disabled=no

:global myVar 42
:local name "MikroTik"
:set myVar 100

/ip firewall filter
add chain=input action=accept comment="Allow established"

:put "Hello $name, value is $myVar"
:log info "Script completed"

/interface bridge port add bridge=bridge1 interface=ether2

:delay 5s
:error "Something went wrong"
:return true

:foreach entry in=[/ip route find] do={
  :put [/ip route get $entry gateway]
}

:for counter from=1 to=10 step=2 do={
  :put $counter
}
