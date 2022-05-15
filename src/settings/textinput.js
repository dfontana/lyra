import './textinput.scss';
export default function TextInput({ id, label, placeholder, onChange, value, children }) {
  return (
    <span className="input-container">
      <label htmlFor={id} className="input-label">
        {label}
      </label>
      <input
        id={id}
        className="input-form"
        placeholder={placeholder}
        type="text"
        autoCorrect="off"
        onChange={onChange}
        value={value}
      />
      {children}
    </span>
  );
}

function SubmitTextInput({ buttonLabel, onClick, isValid, ...rest }) {
  return (
    <TextInput {...rest}>
      <button
        className={`input-submit ${isValid ? '' : 'failed'}`}
        onClick={onClick}
        disabled={!isValid}
      >
        {buttonLabel}
      </button>
    </TextInput>
  );
}

export { SubmitTextInput };
