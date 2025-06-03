import { useState, useRef, useEffect } from "react"
import { motion, useAnimation } from "framer-motion"
import { AlertCircle } from "lucide-react"
import { useNavigate } from "react-router-dom"

interface FormData {
  link: string;
  topic: string
  description: string
  uploader: string
}

interface FormErrors {
  link?: string
  topic?: string
  description?: string
  uploader?: string
}

export default function UploadForm() {
  const [formData, setFormData] = useState<FormData>({
    link: "",
    topic: "",
    description: "",
    uploader: "",
  })

  const [errors, setErrors] = useState<FormErrors>({})
  const [successMessage, setSuccessMessage] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [focusedField, setFocusedField] = useState<keyof FormData | null>(null)

  const controls = useAnimation()
  const formRef = useRef<HTMLFormElement>(null)
  const linkInputRef = useRef<HTMLInputElement>(null)
  const navigate = useNavigate()

  // Focus first input on mount
  useEffect(() => {
    if (linkInputRef.current) {
      linkInputRef.current.focus()
    }
  }, [])

  // Validate form data
  const validate = (): boolean => {
    const newErrors: FormErrors = {}

    if (!formData.link) {
      newErrors.link = "Link is required"
    } else if (!/^https?:\/\/\S+$/.test(formData.link)) {
      newErrors.link = "Enter a valid URL starting with http:// or https://"
    }

    if (!formData.topic) {
      newErrors.topic = "Topic is required"
    }

    if (!formData.description) {
      newErrors.description = "Description is required"
    } else if (formData.description.length < 10) {
      newErrors.description = "Description should be at least 10 characters"
    }

    if (!formData.uploader) {
      newErrors.uploader = "Your name is required"
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  // Handle input changes
  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    const { name, value } = e.target
    setFormData((prev) => ({ ...prev, [name]: value }))

    // Clear field error when typing
    if (errors[name as keyof FormErrors]) {
      setErrors((prev) => ({ ...prev, [name]: undefined }))
    }
  }

  // Handle form submission
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!validate()) {
      controls.start({
        x: [0, -10, 10, -10, 10, 0],
        transition: { duration: 0.5 },
      })
      return
    }

    setIsSubmitting(true)

    try {
      const response = await fetch('/api/links', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
        body: JSON.stringify({
          url: formData.link,
          title: formData.topic,
          description: formData.description,
          uploader_name: formData.uploader,
        }),
      })

      if (!response.ok) {
        throw new Error('Failed to submit link')
      }

      setSuccessMessage("Link submitted successfully!")
      setFormData({ link: "", topic: "", description: "", uploader: "" })
      
      // Redirect to admin dashboard after successful submission
      setTimeout(() => {
        navigate('/admin')
      }, 2000)
    } catch (error) {
      setErrors({ link: "Failed to submit link. Please try again." })
      controls.start({
        x: [0, -10, 10, -10, 10, 0],
        transition: { duration: 0.5 },
      })
    } finally {
      setIsSubmitting(false)
    }
  }

  // Animation variants
  const formVariants = {
    hidden: { opacity: 0, y: 20 },
    visible: {
      opacity: 1,
      y: 0,
      transition: {
        duration: 0.5,
        staggerChildren: 0.1,
      },
    },
  }

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    visible: {
      opacity: 1,
      y: 0,
      transition: { duration: 0.3 },
    },
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-purple-50 via-white to-pink-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900 px-4 py-12">
      <div className="w-full max-w-lg">
        <motion.div
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-center mb-8"
        >
          <h1 className="text-3xl font-bold bg-gradient-to-r from-purple-600 to-pink-500 bg-clip-text text-transparent">
            Share a Link
          </h1>
          <p className="mt-2 text-gray-600 dark:text-gray-400">
            Help others discover valuable resources
          </p>
        </motion.div>

        <motion.form
          ref={formRef}
          className="space-y-6"
          onSubmit={handleSubmit}
          animate={controls}
          variants={formVariants}
          initial="hidden"
          exit="hidden"
        >
          <motion.div variants={itemVariants}>
            <label
              className={`block mb-1.5 font-medium ${
                errors.link ? "text-red-500" : "text-gray-700 dark:text-gray-300"
              }`}
            >
              Link URL
            </label>
            <div className={`relative ${focusedField === "link" ? "ring-2 ring-purple-500 ring-opacity-50" : ""}`}>
              <input
                ref={linkInputRef}
                type="text"
                name="link"
                value={formData.link}
                onChange={handleChange}
                onFocus={() => setFocusedField("link")}
                onBlur={() => setFocusedField(null)}
                className={`w-full px-4 py-3 rounded-lg bg-white dark:bg-gray-800 border ${
                  errors.link
                    ? "border-red-300 dark:border-red-700 bg-red-50 dark:bg-red-900/10"
                    : "border-gray-300 dark:border-gray-700"
                } focus:outline-none transition-colors duration-200`}
                placeholder="https://example.com"
              />
            </div>
            {errors.link && (
              <p className="mt-1.5 text-sm text-red-500 flex items-center">
                <AlertCircle size={14} className="mr-1" />
                {errors.link}
              </p>
            )}
          </motion.div>

          <motion.div variants={itemVariants}>
            <label
              className={`block mb-1.5 font-medium ${
                errors.topic ? "text-red-500" : "text-gray-700 dark:text-gray-300"
              }`}
            >
              Topic
            </label>
            <div className={`relative ${focusedField === "topic" ? "ring-2 ring-purple-500 ring-opacity-50" : ""}`}>
              <input
                type="text"
                name="topic"
                value={formData.topic}
                onChange={handleChange}
                onFocus={() => setFocusedField("topic")}
                onBlur={() => setFocusedField(null)}
                className={`w-full px-4 py-3 rounded-lg bg-white dark:bg-gray-800 border ${
                  errors.topic
                    ? "border-red-300 dark:border-red-700 bg-red-50 dark:bg-red-900/10"
                    : "border-gray-300 dark:border-gray-700"
                } focus:outline-none transition-colors duration-200`}
                placeholder="e.g., Programming, Design, Marketing"
              />
            </div>
            {errors.topic && (
              <p className="mt-1.5 text-sm text-red-500 flex items-center">
                <AlertCircle size={14} className="mr-1" />
                {errors.topic}
              </p>
            )}
          </motion.div>

          <motion.div variants={itemVariants}>
            <label
              className={`block mb-1.5 font-medium ${
                errors.description ? "text-red-500" : "text-gray-700 dark:text-gray-300"
              }`}
            >
              Description
            </label>
            <div
              className={`relative ${focusedField === "description" ? "ring-2 ring-purple-500 ring-opacity-50" : ""}`}
            >
              <textarea
                name="description"
                value={formData.description}
                onChange={handleChange}
                onFocus={() => setFocusedField("description")}
                onBlur={() => setFocusedField(null)}
                rows={4}
                className={`w-full px-4 py-3 rounded-lg bg-white dark:bg-gray-800 border ${
                  errors.description
                    ? "border-red-300 dark:border-red-700 bg-red-50 dark:bg-red-900/10"
                    : "border-gray-300 dark:border-gray-700"
                } focus:outline-none transition-colors duration-200 resize-none`}
                placeholder="Briefly describe what this resource is about and why it's valuable"
              />
            </div>
            {errors.description && (
              <p className="mt-1.5 text-sm text-red-500 flex items-center">
                <AlertCircle size={14} className="mr-1" />
                {errors.description}
              </p>
            )}
          </motion.div>

          <motion.div variants={itemVariants}>
            <label
              className={`block mb-1.5 font-medium ${
                errors.uploader ? "text-red-500" : "text-gray-700 dark:text-gray-300"
              }`}
            >
              Your Name
            </label>
            <div
              className={`relative ${focusedField === "uploader" ? "ring-2 ring-purple-500 ring-opacity-50" : ""}`}
            >
              <input
                type="text"
                name="uploader"
                value={formData.uploader}
                onChange={handleChange}
                onFocus={() => setFocusedField("uploader")}
                onBlur={() => setFocusedField(null)}
                className={`w-full px-4 py-3 rounded-lg bg-white dark:bg-gray-800 border ${
                  errors.uploader
                    ? "border-red-300 dark:border-red-700 bg-red-50 dark:bg-red-900/10"
                    : "border-gray-300 dark:border-gray-700"
                } focus:outline-none transition-colors duration-200`}
                placeholder="Your name"
              />
            </div>
            {errors.uploader && (
              <p className="mt-1.5 text-sm text-red-500 flex items-center">
                <AlertCircle size={14} className="mr-1" />
                {errors.uploader}
              </p>
            )}
          </motion.div>

          <motion.button
            type="submit"
            disabled={isSubmitting}
            className={`w-full py-3 px-4 bg-purple-600 text-white rounded-lg font-medium ${
              isSubmitting
                ? "opacity-50 cursor-not-allowed"
                : "hover:bg-purple-700 active:bg-purple-800"
            } transition-colors duration-200`}
            variants={itemVariants}
          >
            {isSubmitting ? "Submitting..." : "Submit Link"}
          </motion.button>

          {successMessage && (
            <motion.p
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              className="text-center text-green-500 dark:text-green-400 mt-4"
            >
              {successMessage}
            </motion.p>
          )}
        </motion.form>
      </div>
    </div>
  )
}
