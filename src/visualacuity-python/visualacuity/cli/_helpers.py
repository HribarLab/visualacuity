def as_main(parser: "ArgumentParser"):
    """ A little helper to add argparse and a __main__ function to a script """
    def wrapped(main_func):
        if main_func.__module__ == '__main__':
            import __main__, importlib
            scope = importlib.__import__(__main__.__spec__.name, globals(), fromlist=("*", ))
            getattr(scope, main_func.__name__)(**vars(parser.parse_args()))
        return main_func
    return wrapped