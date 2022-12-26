

// Clickjacking defense, break out of frames.
// If JavaScript is disabled, the site will not display at all.
if (self !== top) {
	throw top.location = self.location
}

var app = LiteJS({
	home: "#home",
	on: {
		up: function() {
			app.show(true)
		},
		show: function(params, view) {
			scroll(0, 0)
		}
	},
	ping: {
		"#private": function() {
			if (!El.data.user && app.route != "#login") {
				return app("#login").show()
			}
		}
	}
})

El.data._ = window._ = i18n

!function(window, document, navigator, sessionStorage) {
	var html = document.documentElement
	, body = document.body
	, base = location.href.split("#")[0].replace(/\/[^\/]+$/, "")


	function apiGet() {
		xhrReq("GET", "/api", function(err, data) {
			if (err) return logout(data)
			El.data.user = "User"
			El.data.api = data
			app.show(true)
		})
	}
	function xhrReq(method, url, next, data) {
		var req = xhr(method, url, function onResponse(err, txt) {
			var body = (
				err ? { message: txt, code: err } :
				txt && req.getResponseHeader("Content-Type") == "application/json" ?
				JSON.parse(txt) :
				""
			)
			if (next && next(err, body, req) === true) return
			if (err) {
				app.emit(
					app._e["x" + err] ? "x" + err : "xerr",
					body, method, url, data, onResponse, xhrSend
				)
			}
		})
		xhrSend(req, data)
		return req
	}
	function xhrSend(req, data) {
		if (sessionStorage.auth) {
			req.setRequestHeader("Authorization", sessionStorage.auth)
		}
		if (data) {
			req.setRequestHeader("Content-Type", "application/json")
		}
		req.send(data ? JSON.stringify(data) : null)
	}
	function logout(data) {
		El.data.user = El.data.api = sessionStorage.auth = ""
		app.show(true, {
			message: data.message || data.code ? _("err" + data.code) : ""
		})
	}

	El.bindings.api = function(el, url, attr) {
		var scope = El.scope(el, {})

		if (!scope.c) {
			scope.i = 0
			scope.c = document.createComment("xhr")
			scope.t = el.replaceChild(scope.c, el.firstChild)
		}

		if (!scope.x) {
			scope.x = 1
			var arr = JSON.get(El.data, el.action.slice(base.length))

			for (; scope.i > 0; scope.i--) {
				El.kill(scope.c.previousSibling)
			}
			if (!scope.c.parentNode) return
			if (Array.isArray(arr)) arr = arr.map(clone, arr)
			else if (arr && arr.constructor === Object) {
				arr = Object.keys(arr).map(function(key) {
					return clone(arr[key], key)
				}, arr)
			}
			El.append(el, arr, scope.c)
		}
		function clone(val, i, arr) {
			var e = scope.t.cloneNode(true)
			, sc = El.scope(e, {i: i, item: val})
			sc.i = i
			sc.item = val
			El.render(e)
			scope.i++
			return e
		}
	}

	app.on("login", function(e, data) {
		sessionStorage.auth = "Basic " + btoa(data.user + ":" + data.pass)
		apiGet()
	})
	app.on("logout", logout)
	app.on("patch", function(ev, el, nextView) {
		var data = El.val(el)
		console.log("save", data)
		xhrReq("PATCH", el.action, apiGet, data)
		return Event.stop(ev)
	})
	app.on("upload", function(e, el, view) {
		var data = new FormData(el)
		, prog = El("upload-progress")
		, req = xhr(el.method, el.action, function(err, body) {
			reset()
			if (view) app(view).show({message: body})
		})
		req.onabort = req.onerror = req.ontimeout = reset
		if (req.upload) {
			req.upload.onprogress = progress
		}
		if (sessionStorage.auth) {
			req.setRequestHeader("Authorization", sessionStorage.auth)
		}
		req.send(data)
		el.parentNode.replaceChild(prog, el)
		progress({cancel: cancel})
		function progress(e) {
			El.render(prog, e)
		}
		function cancel() {
			req.abort()
		}
		function reset() {
			if (!prog.parentNode) return
			prog.parentNode.replaceChild(el, prog)
			var prev = el.previousSibling
			El.scope(prev).x = 0
			El.render(prev)
			el.reset()
			app.blur()
		}
		return Event.stop(e)
	})


	_.def({
		"en": "In English"
	})
	_.add("en", {
		"err401": "Unauthorized"
	})

	function setLang(lang) {
		html.lang = Date._locale = _.use(lang)
	}
	app.setLang = setLang
	setLang(_.detect())

	apiGet()


	xhr.load(El.findAll(body, "script[type='litejs/view']").pluck("src"), function() {
		// Start a router to show views
		history.start(app.show)
	})

	El.on(body, "click", function(e) {
		var el = e.target
		, link = !(e.altKey || e.shiftKey) && el.tagName == "A" && el.href.split("#")

		if (link && link[0] == (location.href.split("#")[0])) {
			if (e[El.kbMod]) window.open(el.href, "_blank")
			else if (!history.setUrl(link[1])) app.show(true)
			return Event.stop(e)
		}
	})

}(this, document, navigator, sessionStorage)


