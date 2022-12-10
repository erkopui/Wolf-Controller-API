
El.bindings.xhr = function(el, url, attr) {
	var scope = El.scope(el, {})

	if (!scope.c) {
		scope.i = 0
		scope.c = document.createComment("xhr")
		scope.t = el.replaceChild(scope.c, el.firstChild)
	}

	if (!scope.x) {
		scope.x = 1
		xhr.get(url || el.action, function(err, body) {
			var arr
			for (; scope.i > 0; scope.i--) {
				El.kill(scope.c.previousSibling)
			}
			if (!scope.c.parentNode) return
			if (err) {
				arr = El(".is-red")
				El.txt(arr, "Error:"+err)
			} else {
				arr = attr ? body[attr] : body
				if (Array.isArray(arr)) arr = arr.map(clone, arr)
				else if (arr && arr.constructor === Object) {
					arr = Object.keys(arr).map(function(key) {
						return clone(arr[key], key)
					}, arr)
				}
			}
			El.append(el, arr, scope.c)
		})
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
El.bindings.poll = function(el, url, ms) {
	var tick
	El.on(el, "kill", function() {
		clearTimeout(tick)
	})
	load()
	function load() {
		xhr.get(url, function(err, data, req) {
			if (data)
			El.txt(el, " time: " + req.getResponseHeader("Date") + JSON.stringify(data, null, " ").slice(1, -1).replace(/"/g, ""))
			tick = setTimeout(load, ms)
		})
	}
}

El.bindings.conf = function(el, url) {
	var scope = El.scope(el, {})
	El.on(el, "kill", function() {
		clearTimeout(tick)
	})
	xhr.get(url, function(err, data, req) {
		if (err) return app.show(true, data)
		scope.result = data

		El.txt(el, " time: " + req.getResponseHeader("Date") + JSON.stringify(data, null, " ").slice(1, -1).replace(/"/g, ""))
	})
}


