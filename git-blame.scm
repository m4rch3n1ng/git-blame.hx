(#%require-dylib "libgit_blame" (only-in gix::discover gix/blame blame/default-format blame/format))
(require (only-in "helix/static.scm" cx->current-file get-current-line-number current-directory))
(require (only-in "helix/misc.scm" set-status!))

(define current-pwd (current-directory))
(define current-repo (gix::discover current-pwd))

(define (repo)
  (define pwd (current-directory))
  (if (equal? pwd current-pwd)
      current-repo
      (begin
        (set! current-pwd pwd)
        (set! current-repo (gix::discover pwd))
        current-repo)))

(provide blame/default-format
         blame/format)

(define default-format (blame/default-format))
(define (blame/line file line #:format [format default-format])
  (let ([format (if (string? format)
                    (blame/format format)
                    format)])
    (set-status! (gix/blame (repo) format file line))))

(provide blame/line)

(define (blame/current #:format [format default-format])
  (define file (cx->current-file))
  (define line (get-current-line-number))
  (blame/line file line #:format format))

(provide blame/current)
