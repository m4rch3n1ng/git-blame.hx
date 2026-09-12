(#%require-dylib "libgit_blame" (only-in gix::discover gix/blame blame/default-format blame/format))
(require (only-in "helix/static.scm" cx->current-file get-current-line-number current-directory))
(require (only-in "helix/misc.scm" set-status!))

(define repo (gix::discover))

(provide blame/default-format
         blame/format)

(define default-format (blame/default-format))
(define (blame/line file line #:format [format default-format])
  (set-status! (gix/blame repo format file line)))

(provide blame/line)

(define (blame/current #:format [format default-format])
  (define file (cx->current-file))
  (define line (get-current-line-number))
  (blame/line file line #:format format))

(provide blame/current)
