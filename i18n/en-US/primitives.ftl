name = Name

error-name-too-short = {$min ->
    [one] {name} must be at least {$min} character long
   *[other] {name} must be at least {$min} characters long
}

error-name-too-long = {$max ->
    [one] {name} cannot be more than {$max} character long
   *[other] {name} cannot be more than {$max} characters long
}

error-name-invalid-chars = {name} may only contain letters, hyphens, apostrophes and whitespace

email = Email address

error-invalid-email = Given value is not a valid email address

error-invalid-key = Given value is not a valid key

error-email-too-long = {$max ->
    [one] {email} cannot be more than {$max} character long
   *[other] {email} cannot be more than {$max} characters long
}

password = Password

error-password-too-short = {$min ->
    [one] {password} must be at least {$min} character long
   *[other] {password} must be at least {$min} characters long
}

error-password-too-long = {$max ->
    [one] {password} cannot be more than {$max} character long
   *[other] {password} cannot be more than {$max} characters long
}

error-password-no-lowercase = {password} must contain at least one lowercase letter

error-password-no-uppercase = {password} must contain at least one uppercase letter

error-password-no-digits = {password} must contain at least one digit

error-password-no-special = {password} must contain at least one special character

error-grade-invalid-format = Enter a value from {$min} to {$max} with optional {$fraction-digits} digit fraction
