%css
	.Confirm {
		z-index: 9;
	}
	.Confirm-bg {
		backdrop-filter: blur(5px);
		background-color: rgba(0, 0, 0, .6);
	}
	.Confirm-content {
		position: absolute;
		left: 0;
		right: 0;
		margin: 0 auto;
		top: 4%;
		width: 400px;
		background-color: #fff;
		box-shadow: 0 2px 10px 2px rgba(255,255,255,.5);
	}
	.sm .Confirm-content {
		width: 94%;
	}
	.Confirm--blur {
		transform: scale(.85);
		transform-origin: 50% 100vh;
	}

// reverse animation: x1, y1, x2, y2 -> (1 - x2), (1 - y2), (1 - x1), (1 - y1)
// https://developer.mozilla.org/en-US/docs/Web/API/Notification
// https://developers.google.com/web/fundamentals/push-notifications/display-a-notification
// var n = new Notification(title, options);
// {
//   "//": "Visual Options",
//   "body": "Did you make a $1,000,000 purchase at Dr. Evil...",
//   "icon": "images/ccard.png",  // 192px or more is a safe bet
//   "image": "<URL String>",     // width 1350px or more, ratio of 4:3 for desktop and Android will crop the image
//   "badge": "<URL String>", // 72px or more should be good
//   "vibrate": "<Array of Integers>",
//   "sound": "<URL String>",
//   "dir": "<String of 'auto' | 'ltr' | 'rtl'>",
//   "//": "Behavioural Options",
//   "tag": "<String>", // group messages so that any old notifications that are currently displayed will be closed if they have the same tag as a new notification.
//   "data": "<Anything>",
//   "requireInteraction": "<boolean>", // for Chrome on desktop
//   "renotify": "<Boolean>",
//   "silent": "<Boolean>",
//   "//": "Both Visual & Behavioural Options",
//   "actions": "<Array of Strings>",
//   "//": "Information Option. No visual affect.",
//   "timestamp": "<Long>" // ms
// }
// Star Wars shamelessly taken from the awesome Peter Beverloo
// https://tests.peter.sh/notification-generator/
//   "vibrate": [500,110,500,110,450,110,200,110,170,40,450,110,200,110,170,40,500]
//   "actions": [
//     { "action": "yes", "title": "Yes", "icon": "images/yes.png" },
//     { "action": "no", "title": "No", "icon": "images/no.png" }
//   ]
%js
	View.on("confirm", function(title, opts, next) {
		View.blur()
		if (!next && typeof opts === "function") {
			next = opts
			opts = null
		}
		var sound, vibrate
		, code = ""
		, el = El("Confirm")
		, scope = El.scope(el, El.data)
		, kbMap = {}
		, body = document.body
		, blurEl = body.lastChild
		Object.assign(scope, opts)
		scope.title = title || "Confirm?"
		if (!scope.actions) scope.actions = [
			{ action: "close", title: "Close" }
		]
		for (var a, i = 0; a = scope.actions[i++]; ) {
			if (typeof a == "string") a = scope.actions[i-1] = {title:a,action:a}
			if (a.key) kbMap[a.key] = resolve.bind(el, el, a.action)
		}
		El.cls(blurEl, "Confirm--blur")
		El.cls(el.lastChild, "Confirm--blur", 0, 1)
		El.append(body, el)
		El.render(el, scope)
		if (scope.code) {
			El.findAll(el, ".js-numpad").on("click", numpad)
			kbMap.backspace = kbMap.del = kbMap.num = numpad
		}
		El.addKb(kbMap, el)
		El.findAll(el, ".js-btn").on("click", resolve)
		View.one("navigation", resolve)
		if (scope.bgClose) El.on(el, "click", resolve)
		El.on(el, "wheel", Event.stop)
		El.on(el.lastChild, "click", Event.stop)
		if (scope.vibrate && navigator.vibrate) {
			vibrate = navigator.vibrate(scope.vibrate)
		}
		if (scope.sound && window.Audio) {
			sound = new Audio(scope.sound)
			sound.play()
		}
		function numpad(e, _num) {
			// Enter pressed on focused element
			if (_num == void 0 && e.clientX == 0) return
			var num = _num == void 0 ? e.target[El.T] : _num
			code += num
			if (num == "CLEAR" || num == "del" || num == "backspace") code = ""
			El.txt(El.find(el, ".js-body"), code.replace(/./g, "•") || opts.body)
			// if (code.length == 4 && id && !sent) next(sent = code, id, resolve, reject)
		}
		function resolve(e, key) {
			if (el) {
				El.kill(el, "transparent")
				El.cls(blurEl, "Confirm--blur", el = 0)
				var action = key || El.attr(this, "data-action")
				if (action && next) {
					if (typeof next === "function") next(action, code)
					else if (typeof next[action] === "function") next[action](code)
					else if (next[action]) View.emit(next[action], code)
				}
				if (vibrate) navigator.vibrate(0)
				if (sound) sound.pause()
			}
		}
	})

%el Confirm
	.Confirm.max.fix.anim
		.Confirm-bg.max.abs
		.Confirm-content.Confirm--blur.grid.p2.anim
			.col.ts3 ;txt:: _(title, map)
			.col.js-body ;txt:: _(body, map)
			.row.js-numpad
				;if: code
				;each: num in [1,2,3,4,5,6,7,8,9,"CLEAR",0]
				.col.w4>.btn {num}
			.col
				.group ;each: action in actions
					.btn.js-btn
						;txt:: _(action.title)
						;class:: "w" + (12/actions.length)
						;nop: this.focus()
						;data: "action", action.action
						;class:: "is-" + action.action, action.action
%css
	.mat-Menu,
	.tooltip {
		font-family: "Roboto", "Helvetica", "Arial", sans-serif;
		border-radius: 2px;
		position: absolute;
		margin: 0;
		opacity: 0;
		transform: scale(0);
		transition: opacity .4s cubic-bezier(0, 0, .2, 1) 0s, transform .2s cubic-bezier(0, 0, .2, 1) 0s;
	}
	.tooltip {
		padding: 8px;
		color: #fff;
		background: #666;
		font-weight: 500;
		max-width: 90%;
		text-align: center;
		pointer-events: none;
		z-index: 8;
	}
	.tooltip[data-pos]:before {
		content: "";
		position: absolute;
		display: block;
		width: .4em;
		height: .4em;
		border: .4em solid transparent;
		border-left-color: #666;
		border-top-color: #666;
	}
	.tooltip[data-pos=top]:before {
		bottom: -.3em;
		left: -.3em;
		margin-left: 50%;
		transform: rotate(225deg);
	}
	.tooltip[data-pos=bottom]:before {
		top: -.3em;
		left: -.3em;
		margin-left: 50%;
		transform: rotate(45deg);
	}
	.tooltip[data-pos=right]:before {
		top: 50%;
		left: -.3em;
		margin-top: -.4em;
		transform: rotate(315deg);
	}
	.tooltip[data-pos=left]:before {
		top: -.3em;
		right: -.3em;
		margin-top: 50%;
		transform: rotate(135deg);
	}
	.mat-Menu {
		padding: 8px 0;
		color: #000;
		background: #fff;
		min-width: 124px;
		max-width: 100%;
		z-index: 7;
	}
	.mat-Menu-item {
		display: block;
		padding: 12px 16px;
		text-decoration: none;
		line-height: 24px;
		cursor: pointer;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.mat-Menu-item:hover {
		background-color: #eee;
	}
	.mat-Menu-item[disabled] {
		color: #bdbdbd;
		background-color: transparent;
		cursor: auto;
	}
	.mat-Menu-item.is-divider {
		border-bottom: 1px solid #ddd;
	}
	.mat-Menu.is-visible,
	.tooltip.is-visible {
		transform: scale(1);
		opacity: 1;
	}
	.waves {
		position: relative;
		overflow: hidden;
	}
	.waves-ripple {
		position: absolute;
		border-radius: 50%;
		background-color: #000;
		opacity: 0;
		transform: scale(0);
		transition: opacity .6s cubic-bezier(0, 0, .2, 1) 0s, transform 0s cubic-bezier(0, 0, .2, 1) .6s;
		pointer-events: none;
	}
	.waves.is-active > .waves-ripple,
	.waves-ripple--play {
		opacity: .4;
		transform: scale(1);
		transition: opacity 1s cubic-bezier(0, 0, .2, 1) 0ms, transform .4s cubic-bezier(0, 0, .2, 1) 0ms;
	}
	.shadow-1 {
		box-shadow: 0 2px 2px 0 rgba(0,0,0,.14),0 3px 1px -2px rgba(0,0,0,.2),0 1px 5px 0 rgba(0,0,0,.12);
	}
	/**
	 *  1. Fix Webkit border-radius cropping for children.
	 */
	.Button,
	.Checkbox,
	.Fab,
	.material {
		font-family: "Roboto", "Helvetica", "Arial", sans-serif;
		font-size: 14px;
		font-weight: 500;
		position: relative;
		text-transform: uppercase;
		transition: all .2s cubic-bezier(.4,0,.2,1);
		perspective: 1px;                                         /* 1 */
	}
	.Button:focus,
	.Fab:focus {
		outline: none;
	}
	.Button,
	.Button:disabled:hover {
		background: transparent;
		border: none;
		min-width: 64px;
		height: 36px;
		line-height: 36px;
		border-radius: 2px;
		padding: 0 16px;
		letter-spacing: 0;
		text-align: center;
	}
	.Button.raised,
	.Button:hover {
		background-color: rgba(158,158,158, 0.20);
	}
	.Button:focus {
		background-color: #ddd;
	}
	.raised:focus:not(:active) {
		box-shadow: 0 0 8px rgba(0,0,0,.18),0 8px 16px rgba(0,0,0,.36);
	}
	.Button:disabled {
		color: #aaa;
		background: transparent;
	}
	.Fab {
		border: none;
		border-radius: 50%;
		font-size: 24px;
		height: 56px;
		line-height: 56px;
		width: 56px;
		padding: 0;
		background: rgba(158,158,158,.2);
	}
	.Button.is-warning,
	.Fab--red {
		color: #fff;
		background-color: #f40;
	}
	.Button.is-warning>.waves-ripple,
	.Fab--red>.waves-ripple {
		background-color: #fff;
	}
	.Fab--red>.waves-ripple--play {
		opacity: .6;
	}
	.raised {
		box-shadow: 0 1px 1.5px 0 rgba(0,0,0,.12),0 1px 1px 0 rgba(0,0,0,.24);
	}
	.Checkbox {
		display: block;
		height: 56px;
		line-height: 56px;
		width: 56px;
	}
	.Checkbox-icon {
		overflow: visible;
		display: block;
		height: 50%;
		width: 50%;
		top: 25%;
		left: 25%;
	}

%js
	!function(View) {
		var menuTarget, menuEl, tipTarget, tipEl, tick, wait
		, ripple = El(".waves-ripple")
		El.near = near
		function near(source, target, x, y, margin) {
			var rect = target.getBoundingClientRect()
			, top  = rect.top
			, left = rect.left
			// svg elements dont have offsetWidth, IE8 does not have rect.width
			, width = rect.width || target.offsetWidth || 0
			, height = rect.height || target.offsetHeight || 0
			if (x == "left") {
				left -= source.offsetWidth + margin
				x = "150%"
			} else if (x == "left-start") {
				left -= margin
				x = "0%"
			} else if (x == "right") {
				left += width + margin
				x = "-50%"
			} else if (x == "right-end") {
				left += width + margin - source.offsetWidth
				x = "100%"
			} else {
				left += (width / 2) - (source.offsetWidth/2)
				x = "50%"
			}
			if (y == "top") {
				top -= margin + source.offsetHeight
				y = " 150%"
			} else if (y == "bottom") {
				top += height + margin
				y = " -50%"
			} else {
				top += (height / 2) - (source.offsetHeight/2)
				y = " 50%"
			}
			left += El.scrollLeft()
			top += El.scrollTop()
			El.css(source, {
				"transform-origin": x + y,
				top: (top < 0 ? 0 : top) + "px",
				left: (left < 0 ? 0 : left) + "px"
			})
		}
		El.on(document.body, "mouseover", onOver)
		El.on(window, "focusin", onOver)
		View.on("show", closeTooltip)
		function onOver(e) {
			var x, y, pos
			, target = e.target
			, text = El.attr(target, "data-tooltip")
			, relTarg = e.relatedTarget || e.fromElement
			// without relTarg is event on click
			if (!relTarg && e.type !== "focusin" || target === tipTarget) return
			if (!text && tipTarget) {
				for (; target = target.parentNode; ) {
					if (target === tipTarget) return
				}
			}
			closeTooltip()
			if (!text) return
			tipEl = openVisible("pre.tooltip", tipTarget = target)
			pos = El.attr(target, "data-tooltip-pos") || "top"
			El.txt(tipEl, text)
			if (pos === "left" || pos === "right") {
				x = pos
			} else {
				y = pos
			}
			El.attr(tipEl, "data-pos", pos)
			near(tipEl, target, x, y, 6)
		}
		function openVisible(tag, target) {
			var el = typeof tag == "string" ? El(tag) : tag
			El.scope(el, El.scope(target))
			El.render(el)
			El.append(document.body, el)
			El.cls(el, "is-visible", 1, 5)
			return el
		}
		function closeVisible(el, delay) {
			if (el) {
				setTimeout(el.closeFn || El.kill.bind(El, el), 999)
				El.cls(el, "is-visible", 0, delay)
			}
		}
		function closeTooltip() {
			if (tipEl) {
				closeVisible(tipEl)
				tipTarget = tipEl = null
			}
		}
		function closeMenu(e) {
			if (e && e.target == menuTarget) return
			if (menuEl) {
				closeVisible(menuEl, 200)
				El.cls(menuTarget, "is-active", menuEl = menuTarget = null)
			}
		}
		View.on("resize", closeMenu)
		View.on("closeMenu", closeMenu)
		View.on("showMenu", function(e, target, menu, x, y, margin) {
			Event.stop(e)
			var close = menuEl && menuTarget == target
			closeMenu()
			if (close) return
			menuEl = openVisible(menu, target)
			if (x == "mouse") {
				El.css(menuEl, {
					top: e.pageY + "px",
					left: e.pageX + "px"
				})
			} else {
				El.cls(menuTarget = target, "is-active")
				near(menuEl, target, x, y, 4)
			}
			if (menuEl.style.transform !== void 0) {
				El.cls(menuEl, "no-events")
				El.on(menuEl, "transitionend", function(e) {
					if (e.propertyName === "transform") El.cls(menuEl, "no-events", 0)
				})
			}
		})
		El.on(document.body, "click", closeMenu)
		El.on(document.body, "pointerdown", pointerdown)
		function pointerdown(e) {
			var target = e.target
			if (!El.hasClass(target, "waves") || target.disabled) return
			var rect = target.getBoundingClientRect()
			, fromMouse = !El.hasClass(target, "Checkbox-icon")
			, top = fromMouse ? e.clientY - rect.top : rect.height
			, left = fromMouse ? e.clientX - rect.left : rect.width
			, maxH = Math.max(top, target.offsetHeight - top)
			, maxW = Math.max(left, target.offsetWidth - left)
			, max = Math.sqrt(maxH * maxH + maxW * maxW)
			, size = (fromMouse ? 2 * max : max) + "px"
			El.css(ripple, {
				top: (top - max) + "px",
				left: (left - max) + "px",
				width: size,
				height: size
			})
			El.append(target, ripple)
			clearTimeout(tick)
			end()
			wait = 1
			tick = setTimeout(end, 800)
			El.one(document.body, "pointerup", end)
			ripple.offsetTop // force repaint
			El.cls(ripple, "waves-ripple--play")
		}
		function end() {
			if (!(wait--)) {
				El.cls(ripple, "waves-ripple--play", 0)
			}
		}
	}(View)


%el Checkbox
	label.Checkbox
		input[type=checkbox].hide
		i.Checkbox-icon.waves

%el Button
	button[type=button].Button.waves Button

%el Fab
	button[type=button].Fab.waves.raised

%el Radio
	button[type=button].Radio.waves

%css
	.MenuBtn {
		position: relative;
		width: 30px;
		height: 30px;
		background: transparent;
		color: #fff;
	}
	.MenuBtn-x,
	.MenuBtn-x:before,
	.MenuBtn-x:after {
		display: block;
		content: "";
		background-color: currentColor;
		position: absolute;
		width: 100%;
		height: .3em;
		border-radius: .3em;
		pointer-events: none;
		transition-property: transform;
	}
	.MenuBtn-x:before {
		transform: translate(0, -.6em);
	}
	.MenuBtn-x:after {
		transform: translate(0, .6em);
	}
	.MenuBtn--back > .MenuBtn-x,
	.MenuBtn--close > .MenuBtn-x {
		color: #666;
		transform: rotateZ(-180deg);
	}
	.MenuBtn--back > .MenuBtn-x:before {
		transform: rotateZ(45deg) scaleX(.75) translate(0, -230%);
	}
	.MenuBtn--back > .MenuBtn-x:after {
		transform: rotateZ(-45deg) scaleX(.75) translate(0, 230%)
	}
	.MenuBtn--close > .MenuBtn-x {
		background-color: transparent;
	}
	.MenuBtn--close > .MenuBtn-x:before {
		transform: rotateZ(45deg) translate(0, 0);
	}
	.MenuBtn--close > .MenuBtn-x:after {
		transform: rotateZ(-45deg) translate(0, 0);
	}

%el MenuBtn
	button[type=button].MenuBtn.reset.noselect
		.MenuBtn-x.anim
%css
	::-moz-focus-inner {
		border: 0;
		padding: 0;
	}
	.Form1-del {
		display: block;
		margin: -10px -10px 0 0;
		font-size: 20px;
		font-weight: 700;
		opacity: .2;
		border: 1px solid transparent;
		line-height: 16px;
		width: 20px;
		height: 20px;
		text-align: center;
		border-radius: 4px;
	}
	.Form1-del:hover {
		opacity: 1;
		border: 1px solid #aaa;
		background-image: linear-gradient(to bottom, #ddd, #888);
	}
	/**
	 *  1. avoid ios styling the submit button
	 */
	.input {
		display: block;
		border-radius: 4px;
		border: 1px solid #aaa;
		overflow: auto;
	}
	.field {
		width: 100%;
	}
	.btn,
	input,
	select,
	textarea {
		display: block;
		border-radius: 4px;
		border: 1px solid #aaa;
		font-size: 14px;
		font-weight: 400;
		line-height: 30px;
		height: 32px;
		padding: 0 8px;
		margin: 0;
	}
	input[type=checkbox] {
		height: auto;
	}
	input[type=time] {
		padding: 0 0 0 8px;
	}
	textarea {
		height: 64px;
		padding: 8px;
		margin: 0;
		line-height: 1.1;
	}
	select {
		padding-right: 0;
	}
	select[multiple] {
		height: auto;
		padding: 0;
	}
	input[type=radio],
	input[type=checkbox] {
		width: auto;
		display: inline;
		margin-top: -2px;
	}
	.btn,
	input[type=submit] {                /* 1 */
		-webkit-appearance: none;   /* 1 */
		position: relative;
		padding: 0px 14px;
		text-align: center;
		text-decoration: none;
		/* default look */
		background-color: #ddd;
		color: #444;
		cursor: pointer;
	}
	option[disabled],
	.btn.disabled,
	.btn[disabled] {
		box-shadow: none;
		cursor: not-allowed;
		font-style: italic;
		opacity: .6;
		pointer-events: none;
	}
	.group > .btn {
		border-radius: 0;
		margin-left: -1px;
		float: left;
	}
	.group > .btn:first-child {
		border-top-left-radius: 4px;
		border-bottom-left-radius: 4px;
	}
	.group > .btn:last-child {
		border-top-right-radius: 4px;
		border-bottom-right-radius: 4px;
	}
	.btn--narrow {
		line-height: 1.6;
		margin: .7em 0;
	}
	.btn__spacer {
		height: 33px;
	}
	.md .input__label,
	.lg .input__label {
		padding-right: 8px;
		text-align: right;
		line-height: 28px;
	}
	.input__hint {
		text-align: right;
		color: #444;
	}
	input[type=checkbox]+.input__hint {
		display: inline-block;
		margin-left: 8px;
	}
	.btn:active, .btn:focus,
	input:active, input:focus,
	select:active, select:focus,
	textarea:active, textarea:focus {
		border-color: #257;
		outline: 0 none;
		box-shadow:
			0 2px 5px rgba(0, 0, 0, .5) inset,
			0 0 2px 2px #6ae;
		z-index: 1;
	}
	.btn:hover,
	.btn:focus {
		background-color: #eee;
		color: #333;
		text-decoration: none;
	}
	.btn:active,
	.btn.is-active {
		background-color: #ccc;
		box-shadow: inset 0 0 8px rgba(0, 0, 0, .5);
	}


%el form1-row
	label.row
		.col.md-w4.input__label
			= _(title||name)
		.col.md-w8
			%child
			.input__hint
				= _(description)
				;if: description

%el form1-subheader
	.col
		= _(title)

%el form1-fieldset
	fieldset.grid.b2
		legend
			= _(schema.title || _link.title || "")

%el form1
	form1-row
		input.field

%el form1-ro
	form1-row>span ;txt: value

%el form1-hidden
	div>input.field[type=hidden]

%el form1-boolean
	form1-row
		input.field[type=checkbox] ;value: value

%el form1-boolean-ro
	form1-row>span
		= _(!!value)

%el form1-password
	form1-row
		input.field[type=password]

%el form1-new-password
	form1-row
		input.field[type=password][autocomplete=new-password]

%el form1-text
	form1-row
		textarea.field

%el form1-text-ro
	form1-ro

%el form1-enum
	form1-row
		select.field ;each:val in data["enum"]
			option
				;val:: val
				= _("" + val)

%el form1-enum-ro
	form1-ro

%el form1-list
	form1-row
		select.field
			;list: api(resourceCollection.format(data.params, data)), required ? 0 : [""], value
			option
				;val:: item.id
				;txt:: _(item.name)

%el form1-list-ro
	form1-row>span
		= _(item.name)

%el form1-array
	.col
		.input.p13
			.left
				= _(title||name)
			.input__hint
				= _(description)
			.js-items.cf
			a.btn.right
				;if: !data.noAdd
				;txt: _(data.name + ".Add")
				@click: data.add

%el form1-array-item
	.input.p3.m2b.js-del
		a.right.Form1-del.hand ×
			;if: !data.noAdd
			;on: "click", data.del
		b
			;if: title
			;txt: title
		.grid.b2.js-item
%css
	.Slider {
		width: 200px;
		background: transparent;
		touch-action: none;
	}
	.Slider.is-vertical {
		width: auto;
		height: 200px;
	}
	.is-vertical > .Slider-track {
		margin: 0 14px;
		width: 4px;
		height: 100%;
	}
	.is-vertical > .Slider-track > .Slider-fill {
		top: auto;
		bottom: 0;
		width: 4px;
	}
	.is-vertical > .Slider-track > .Slider-fill > .Slider-knob {
		margin: -10px -8px 0 0;
	}
	.Slider-track {
		position: relative;
		margin: 14px 0;
	}
	.Slider-track,
	.Slider-fill {
		height: 4px;
		border-radius: 2px;
		overflow: visible;
		background: #666;
	}
	.Slider-fill {
		background: rgba(255,255,255,.57);
	}
	.Slider-knob,
	.Toggle-knob {
		position: relative; /* for IE6 overflow:visible bug */
		width: 20px;
		height: 20px;
		border-radius: 50%;
		box-shadow:
			0 1px 4px rgba(0, 0, 0, .2);
	}
	.Slider-knob {
		float: right;
		margin: -8px -10px 0 0;
		outline: none;
		background: #f5f5f5;
		background-color: rgb(245, 245, 245);
	}
	.Slider-knob:hover,
	.Slider-knob:focus,
	:hover>.Toggle-knob {
		box-shadow:
			0 0 0 8px rgb(0, 0, 0, .2),
			0 1px 4px rgba(0, 0, 0, .3);
	}
	.Slider-knob.is-active {
		box-shadow:
			0 0 0 12px rgb(0, 0, 0, .2),
			0 1px 5px 5px rgba(0, 0, 0, .3);
	}
	.Slider-knob.is-active:before,
	.Slider-knob.is-active:after {
		position: absolute;
		width: 32px;
		height: 32px;
		left: -6px;
		display: block;
		top: -44px;
		animation: .1s linear 0s 1 forwards Slider-active;
	}
	.Slider-knob.is-active:before {
		content: "";
		border-radius: 50% 50% 50% 0;
		transform: rotate(-45deg);
		background: inherit;
		box-shadow:
			0 1px 4px rgba(0, 0, 0, .2);
	}
	.Slider-knob.is-active:after {
		content: attr(data-val);
		color: #000;
		font-size: 14px;
		line-height: 32px;
		text-align: center;
	}
	.Toggle {
		position: relative;
		display: block;
		width: 36px;
		height: 22px;
		-webkit-tap-highlight-color: rgba(0, 0, 0, 0);
	}
	.Toggle:before {
		display: block;
		content: "";
		background: #bdbdbd;
		position: absolute;
		width: 36px;
		height: 14px;
		top: 4px;
		border-radius: 7px;
		-webkit-tap-highlight-color: rgba(0, 0, 0, 0);
	}
	.Toggle-knob {
		background-color: #666;
		top: 1px;
		left: 0px;
	}
	input:checked + .Toggle-knob {
		background-color: #00a651;
		left: 16px;
	}
	@keyframes Slider-active {
		0% {
			top: 0px;
			opacity: 0;
		}
		to {
			top: -44px;
			opacity: 1;
		}
	}
	/*
	.Slider.color .Slider-fill {
		background: red;
	}
	.Slider.color .Slider-fill+.Slider-fill {
		background: green;
	}
	.Slider.color .Slider-fill+.Slider-fill+.Slider-fill {
		background: blue;
	}
	.Slider.no-first > .Slider-track > .Slider-fill:last-child {
		background: #666;
	}
	*/

%js
	var on = El.on
	, off = El.off
	El.bindings.SliderInit = function(el) {
		var knobLen, offset, px, drag, min, max, step, minPx, maxPx, value
		, vert = El.hasClass(el, "is-vertical")
		, track = el.firstChild
		, fill = track.firstChild
		, knob = fill.lastChild
		, emit = El.emit.bind(el, el, "change").rate(500, true)
		on(window, "blur", stop)
		on(el, "pointerdown", start)
		el.val = set
		setTimeout(function() { set(value||0) }, 10)
		function load() {
			var attr = vert ? "offsetHeight" : "offsetWidth"
			, range = (El.attr(el, "range") || "").split(/[^+\-\d.]/) // min:max:step:margin
			min = +(range[0] || 0)
			max = +(range[1] || 100)
			step = +(range[2] || 1)
			knobLen = knob[attr]>>1
			minPx = 0
			maxPx = track[attr] - knobLen - knobLen
			px = maxPx / (max - min)
		}
		function start(e) {
			drag = true
			load()
			var tmp = el.getBoundingClientRect()
			offset = (vert ? tmp.top + maxPx + El.scrollTop() + knobLen : tmp.left + El.scrollLeft()) + knobLen
			tmp = offset - e.clientX + (value-min||0)*px
			if (tmp < knobLen && tmp > -knobLen) offset -= tmp
			if (track.childNodes.length > 1) {
				fill = track.firstChild
				var next
				, x = maxPx
				, tmp = fill
				, diff = vert ? offset - e.pageY : e.pageX - offset
				for (; tmp; tmp = tmp.nextSibling) {
					next = diff - tmp[attr] + knobLen
					if (next < 0 ? -next <= x : next < x) {
						fill = tmp
						knob = fill.firstChild
						x = next < 0 ? -next : next
					}
				}
				if (fill.previousSibling) {
					maxPx = fill.previousSibling[attr] - knobLen
					if (range[3]) maxPx -= px * range[3]
				}
				if (fill.nextSibling) {
					minPx = fill.nextSibling[attr] - knobLen
					if (range[3]) minPx += px * range[3]
				}
			}
			move(e)
			listen(on)
		}
		function move(e) {
			var diff = vert ? offset - e.pageY : e.pageX - offset
			diff = (diff > maxPx ? maxPx : (diff < minPx ? minPx : diff))
			set((diff / px) + min, e, diff)
			return Event.stop(e)
		}
		function stop(e) {
			if (drag) {
				drag = false
				listen(off)
				set(value)
			}
		}
		function listen(on) {
			El.cls(fill, "anim", !drag)
			El.cls(knob, "is-active", drag)
			on(document, "pointerup", stop)
			on(document, "pointermove", move)
		}
		function set(val, e, pos) {
			load()
			val = (val < min ? min : val > max ? max : val).step(step)
			if (value !== void 0 && (!drag || pos !== void 0)) {
				El.css(fill, vert ? "height" : "width", ((pos || (val-min)*px)+knobLen) + "px", 0)
			}
			if (value !== val) {
				el.value = value = val
				if (drag && e) emit(e)
				var format = El.attr(el, "format")
				El.attr(knob, "data-val", format ? _(format, {val:val}) : val)
			}
		}
	}
	El.bindings.SliderInit.once = 1

%el Slider
	button.Slider.reset ;SliderInit
		.Slider-track
			.Slider-fill.abs.anim
				.Slider-knob.anim[tabindex=0]

/%el Slider2
/	button.Slider.reset ;SliderInit
/		.Slider-track
/			.Slider-fill.abs.anim
/				.Slider-knob.anim[tabindex=0]
/			.Slider-fill.abs.anim
/				.Slider-knob.anim[tabindex=0]
/
/%el Slider3
/	button.Slider.reset ;SliderInit
/		.Slider-track
/			.Slider-fill.abs.anim
/				.Slider-knob.anim[tabindex=0]
/			.Slider-fill.abs.anim
/				.Slider-knob.anim[tabindex=0]
/			.Slider-fill.abs.anim
/				.Slider-knob.anim[tabindex=0]

%el Toggle
	label.Toggle.reset[tabindex=0]
		;fixReadonlyCheckbox
		input[type=checkbox].hide
			;readonly: row && !row.write
			;checked: model && !!model.get(row.path)
		.Toggle-knob.anim