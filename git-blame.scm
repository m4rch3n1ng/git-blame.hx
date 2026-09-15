(#%require-dylib
 "libgit_blame"
 (only-in git-blame::discover git-blame::blame git-blame::default-format git-blame::format))
(require (only-in "helix/static.scm" cx->current-file get-current-line-number current-directory))
(require (only-in "helix/misc.scm" set-status!))

(define current-pwd (current-directory))
(define current-repo (git-blame::discover current-pwd))

(define (repo)
  (define pwd (current-directory))
  (if (equal? pwd current-pwd)
      current-repo
      (begin
        (set! current-pwd pwd)
        (set! current-repo (git-blame::discover pwd))
        current-repo)))

(define blame/default-format git-blame::default-format)
(define blame/format git-blame::format)

(provide blame/default-format
         blame/format)

(define default-format (git-blame::default-format))
(define (blame/line file line #:format [format default-format])
  (let ([format (if (string? format)
                    (git-blame::format format)
                    format)])
    (set-status! (git-blame::blame (repo) format file line))))

(define (blame/current #:format [format default-format])
  (define file (cx->current-file))
  (define line (get-current-line-number))
  (blame/line file line #:format format))

(provide blame/line
         blame/current)
