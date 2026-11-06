; 定义不透明颜色的像素格式规则
(define (opaque-formats q)
  (cond ((<= q 50) "pal-rgb")
        (else "rgb24")))

; 定义透明颜色的像素格式规则
(define (transparent-formats q)
  (cond ((<= q 50) "pal-argb-premul")
        (else "argb32-premul")))

; 计算透明和品质参数作用下的目标像素格式
(define pixel-format
  ((if env.transparent
      transparent-formats opaque-formats)
    env.quailty))

; 图片是否转换为查色表格式
(define palette (<= env.quailty 50))

; 是否添加 --verbose 命令行参数
(define if-verbose (if env.verbose "--verbose " ""))

; 调用 pngquant 命令将图片颜色缩减至 256 色以内，系统中需要安装 pngquant
(define color-reduction
  (string-append "pngquant --ext=.png --force " if-verbose env.image-path))

; 转换图片为 PGF 格式
(define convert (string-append "image-forge "
  "--format=" pixel-format " " ; 指定输出像素格式
  "--compress --min-compress-ratio=5 " ; 压缩图像数据减小文件尺寸，最小压缩比为 5
  "--align=16 --pixel-align " ; 图片按 16 像素对齐
  if-verbose
  env.image-path))

; 移除图片 Alpha 通道并添加背景
(define remove-alpha (string-append "image-forge --bypass "
  ; 在手表上，非透明图片可以移除 alpha 通道并用黑色背景混合，这种操作可以提高 PAL 颜色缩减后的图像质量
  (if env.transparent "" "--background black ")
  if-verbose
  env.image-path))

; 将 PGF 图像转回 PNG 的命令
(define decode
  (string-append "image-forge --decode " if-verbose env.image-path))

; 执行一个命令并在 verbose 模式中打印命令内容，如果命令以非 0 异常退出则调用函数 f
(define (execute-try cmd f)
  (begin
    (if env.verbose ; 如果为 verbose 模式则打印命令内容
      (display (string-append "run command: " cmd "\n")))
    (let ((r (system cmd)))
      (if (= r 0) 0 (f r)))
  ))

; 执行一个命令，并会在 verbose 模式打印命令内容，如果命令异常退出则退出程序
(define (execute cmd)
  (execute-try cmd (lambda (x)
    (begin ; 失败时打印错误码并异常退出
      (display (string-append "subprocess failed (" (number->string x) "): " cmd "\n"))
      (exit-fail)
  ))))

; 转换图像
(define (run-convert)
  (begin
    (execute remove-alpha) ; 先移除透明通道
    (if palette (execute color-reduction)) ; 如果是查色表格式则缩减图片的像素数量
    (execute convert) ; 执行图片转换命令
  ))
 
; 定义目标对应的转换策略
(targets env.target
  ; 设备模式：最终用于目标设备的图片转换流程
  ("device" (run-convert))
  ; 模拟器模式：仅移除非透明图片的 alpha 通道，不转换格式
  ("emulator" (execute remove-alpha))
  ; 预处理模式：移除非透明图片的 alpha通道并添加背景
  ("preprocess" (execute remove-alpha))
  ; 预览模式：生成和实际设备显示效果一致的 PNG 预览图片
  ("preview" (begin
    (run-convert) ; 先把图片转换为 PGF 格式
    (execute decode))) ; 再把图片转回 PNG
  )
