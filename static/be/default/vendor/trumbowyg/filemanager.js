/*!
 * RustAdmin Trumbowyg "filemanager" plugin (bootstrap stub).
 *
 * Registers a toolbar button that opens a modal to upload/list/delete images via the media
 * endpoints (`/admin/v1/media/{list,upload,delete}`), reading the CSRF token from
 * <meta name="csrf-token"> and sending it as the `X-CSRF-Token` header. The full modal UI is
 * wired in the media phase; this stub keeps the toolbar button functional and the head
 * <script> load 404-free.
 */
(function ($) {
    'use strict';
    if (!$ || !$.trumbowyg) return;

    function csrf() {
        var m = document.querySelector('meta[name="csrf-token"]');
        return m ? m.getAttribute('content') : '';
    }

    $.extend(true, $.trumbowyg, {
        langs: { en: { filemanager: 'File Manager' } },
        plugins: {
            filemanager: {
                init: function (trumbowyg) {
                    trumbowyg.addBtnDef('filemanager', {
                        title: 'File Manager',
                        ico: 'insertImage',
                        fn: function () {
                            // Minimal flow: prompt for a URL or trigger upload picker.
                            // Replaced by a full modal (list/upload/delete) in the media phase.
                            var input = document.createElement('input');
                            input.type = 'file';
                            input.accept = 'image/*';
                            input.onchange = function () {
                                var file = input.files && input.files[0];
                                if (!file) return;
                                var fd = new FormData();
                                fd.append('file', file);
                                fetch('/admin/v1/media/upload', {
                                    method: 'POST',
                                    headers: { 'X-CSRF-Token': csrf() },
                                    body: fd,
                                    credentials: 'same-origin'
                                })
                                    .then(function (r) { return r.json(); })
                                    .then(function (res) {
                                        if (res && res.data && res.data.url) {
                                            trumbowyg.execCmd('insertImage', res.data.url, false, true);
                                        }
                                    })
                                    .catch(function () { /* surfaced by Toast in full impl */ });
                            };
                            input.click();
                        }
                    });
                }
            }
        }
    });
})(window.jQuery);
