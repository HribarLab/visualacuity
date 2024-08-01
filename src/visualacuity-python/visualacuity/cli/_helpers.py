def as_main(parser: "ArgumentParser"):
    """ A little helper to add argparse and a __main__ function to a script """
    def wrapped(main_func):
        if main_func.__module__ == '__main__':
            import __main__, os, importlib
            module_name, _ = os.path.splitext(os.path.basename(__main__.__file__))
            scope = importlib.__import__(module_name, globals())
            getattr(scope, main_func.__name__)(**vars(parser.parse_args()))
        return main_func
    return wrapped