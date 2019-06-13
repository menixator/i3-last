# i3-last

i3-last is a utility progam that allows a user to jump between i3 windows in the
order that they were last accessed.



## How to Use
- Start i3-last within i3-config
- Add some bindings
```
bindsym $mod+Tab exec --no-startup-id pkill -SIGRTMIN+3 i3-last&
bindsym $mod+Shift+Tab exec --no-startup-id pkill -SIGRTMIN+2 i3-last&
bindsym $mod+b exec --no-startup-id pkill -SIGRTMIN+4 i3-last&
```
