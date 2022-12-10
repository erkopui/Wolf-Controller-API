

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
	// Detect base from HTML <base> element
	, base = (html.getElementsByTagName("base")[0] || _).href
	, body = document.body

	function addMethod(method, name) {
		xhr[name || method.toLowerCase()] = xhr[method] = xhrReq.bind(null, method)
	}
	addMethod("GET")
	addMethod("POST")
	addMethod("PUT")
	addMethod("DELETE", "del")

	app.on("login", function(ev, data) {
		sessionStorage.auth = "Basic " + btoa(data.user + ":" + data.pass)
		up()
	})
	app.on("logout", logout)

	function logout(ev, data) {
		El.data.user = El.data.api = sessionStorage.auth = ""
		app.show(true)
	}
	function up() {
		xhr.get("/api", handleData)
	}
	function handleData(err, data) {
		if (err) {
			//return logout()
			El.data.user = "User"
			El.data.api = {
				net: {
					static: false,
					ip:"1.2.3.4"
				}
			}
		} else {
			El.data.user = "User"
		}
		app.show(true, err && data)
	}
	app.on("upload", function(e, el, view) {
		var data = new FormData(el)
		, prog = El("upload-progress")
		, req = xhr(el.method, el.action, function(err, body) {
			reset()
			if (view) app(view).show({message: body})
		})
		if (sessionStorage.auth) {
			req.setRequestHeader("Authorization", sessionStorage.auth)
		}
		req.onabort = req.onerror = req.ontimeout = reset
		if (req.upload) {
			req.upload.onprogress = progress
		}
		req.send(data)
		el.parentNode.replaceChild(prog, el)
		progress({cancel: cancel})
		function progress(ev) {
			El.render(prog, ev)
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

	_.def({
		"en": "In English"
	})
	function setLang(lang) {
		html.lang = Date._locale = _.use(lang)
	}
	app.setLang = setLang
	setLang(_.detect())

	history.scrollRestoration = "manual"











	xhr.load(El.findAll(body, "script[type='litejs/view']").pluck("src"), function() {
		// Start a router to show views
		history.start(app.show)
	})

	El.on(body, "click", function(e) {
		var el = e.target
		, link = !(e.altKey || e.shiftKey) && el.tagName == "A" && el.href.split("#")

		if (link && link[0] == (base || location.href.split("#")[0])) {
			if (e[El.kbMod]) window.open(el.href, "_blank")
			else if (!history.setUrl(link[1])) app.show(true)
			return Event.stop(e)
		}
	})

}(this, document, navigator, sessionStorage)


